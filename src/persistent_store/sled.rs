use std::io;
use std::mem::size_of;
use std::path::PathBuf;

use async_trait::async_trait;

use super::{Challenge, PersistentStore, SetChallengeError};
use crate::sealed::Sealed;

static SECRETS_TABLE: &[u8] = b"secrets";
static META_TABLE: &[u8] = b"meta";

static COUNTER_ROW: &[u8] = b"counter";
static CHALLENGE_ROW: &[u8] = b"challenge";

#[derive(Clone)]
pub struct SledDB {
    db: sled::Db,
    secrets: sled::Tree,
    meta: sled::Tree,
}

#[async_trait]
impl PersistentStore for SledDB {
    type PublicKey = sled::IVec;
    type SecretShare = sled::IVec;
    type Error = sled::Error;

    async fn open(path: PathBuf) -> sled::Result<Self> {
        let db = sled::open(path)?;
        let secrets = db.open_tree(SECRETS_TABLE)?;
        let meta = db.open_tree(META_TABLE)?;
        Ok(Self { db, secrets, meta })
    }

    async fn add_server_secret_share(
        &self,
        public_key: &[u8],
        server_secret_share: &[u8],
    ) -> sled::Result<()> {
        self.secrets
            .compare_and_swap(public_key, None::<Vec<u8>>, Some(server_secret_share))?
            .map_err(|_| {
                io::Error::new(io::ErrorKind::AlreadyExists, "server share already exist")
            })?;
        self.secrets.flush_async().await?;
        Ok(())
    }

    async fn get_server_secret_share(
        &self,
        public_key: &[u8],
    ) -> sled::Result<Option<Sealed<Self::PublicKey, Self::SecretShare>>> {
        let secret = match self.secrets.get(public_key)? {
            Some(s) => s,
            None => return Ok(None),
        };
        Ok(Some(Sealed::new(sled::IVec::from(public_key), secret)))
    }

    async fn increase_ping_counter(&self) -> sled::Result<u128> {
        let result = self.meta.transaction(|tx| {
            let counter = match tx.get(COUNTER_ROW)? {
                Some(value) => read_counter(value).ok_or(
                    sled::transaction::ConflictableTransactionError::Abort(
                        io::ErrorKind::InvalidData,
                    ),
                )?,
                None => 0,
            };

            tx.insert(COUNTER_ROW, &(counter + 1).to_le_bytes())?;
            tx.remove(CHALLENGE_ROW)?;

            Ok(counter + 1)
        });
        let new_counter = match result {
            Ok(c) => c,
            Err(sled::transaction::TransactionError::Storage(e)) => return Err(e),
            Err(sled::transaction::TransactionError::Abort(e)) => Err(io::Error::from(e))?,
        };
        self.meta.flush_async().await?;

        Ok(new_counter)
    }

    async fn get_ping_counter(&self) -> sled::Result<u128> {
        let value = match self.meta.get(COUNTER_ROW)? {
            Some(c) => c,
            None => return Ok(0),
        };
        read_counter(value).ok_or(
            io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid internal counter representation",
            )
            .into(),
        )
    }

    async fn set_challenge(
        &self,
        challenge: Challenge,
    ) -> Result<(), SetChallengeError<sled::Error>> {
        let serialized = serde_json::to_vec(&challenge)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e).into())
            .map_err(SetChallengeError::Store)?;
        let result = self.meta.transaction(|tx| {
            let counter = match tx.get(COUNTER_ROW)? {
                Some(value) => read_counter(value).ok_or(
                    sled::transaction::ConflictableTransactionError::Abort(
                        SetChallengeError::Store(
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                "invalid internal counter representation",
                            )
                            .into(),
                        ),
                    ),
                )?,
                None => 0,
            };

            if challenge.id < counter {
                return Err(sled::transaction::ConflictableTransactionError::Abort(
                    SetChallengeError::Outdated,
                ));
            } else if challenge.id > counter {
                return Err(sled::transaction::ConflictableTransactionError::Abort(
                    SetChallengeError::MismatchedId,
                ));
            }

            let current_challenge: Option<Challenge> = match tx.get(CHALLENGE_ROW)? {
                Some(c) => serde_json::from_slice(&c)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e).into())
                    .map_err(SetChallengeError::Store)
                    .map_err(sled::transaction::ConflictableTransactionError::Abort)?,
                None => None,
            };

            if let Some(current_challenge) = current_challenge {
                if current_challenge.id == challenge.id {
                    return Err(sled::transaction::ConflictableTransactionError::Abort(
                        SetChallengeError::AlreadySet(current_challenge),
                    ));
                }
            }

            tx.insert(CHALLENGE_ROW, serialized.as_slice())?;

            Ok(())
        });

        match result {
            Ok(()) => {
                self.meta
                    .flush_async()
                    .await
                    .map_err(SetChallengeError::Store)?;
                Ok(())
            }
            Err(sled::transaction::TransactionError::Storage(e)) => {
                Err(SetChallengeError::Store(e))
            }
            Err(sled::transaction::TransactionError::Abort(e)) => Err(e),
        }
    }

    async fn get_challenge(&self) -> sled::Result<Option<Challenge>> {
        let serialized = match self.meta.get(CHALLENGE_ROW)? {
            Some(s) => s,
            None => return Ok(None),
        };
        let challenge: Challenge = serde_json::from_slice(&serialized)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Some(challenge))
    }
}

fn read_counter(value: impl AsRef<[u8]>) -> Option<u128> {
    if value.as_ref().len() != size_of::<u128>() {
        return None;
    }
    let mut counter = [0u8; size_of::<u128>()];
    counter.copy_from_slice(value.as_ref());
    let counter = u128::from_le_bytes(counter);
    Some(counter)
}

#[cfg(test)]
mod persistent_store_should {
    use std::io;

    use super::{PersistentStore, SledDB};
    use crate::persistent_store::{Challenge, SetChallengeError};

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    lazy_static::lazy_static! {
        static ref TEST_CHALLENGE: vdf::UnsolvedVDF = serde_json::from_str(r#"{"x":[1,[1885652591,17533517,2416140196,2102789474,1234557046,817216195,3655015316,1960318755]],"setup":{"t":[1,[1]],"N":[1,[1773823066,1567367735,2844690069,1588019752,2702647890,3059924173,848501649,223024724,2163570840,2072740969,1358934230,1511233973,
            1752724635,151106506,1523033053,1067477923,3213627708,3064750367,2075312732,3562018252,3325444970,1512641256,864984444,2809702737,3651009371,2221401360,275820096,663498737,435288944,1585439220,1588009357,1510728236,940021168,2602478749,3724471822,3448406120,11694078,1826577040,1068252436,4269783695,1368464316,221410714,
            1030199234,3308526525,1260113467,2369328081,3577035636,4031188375,1583697031,3949780996,2720748085,592794227,2159723444,4203311255,3605012052,1627223175,4268320010,284996006,2647917898,581059137,1412522909,565643573,2889868497,1949977675,3743467154,415346208,1343833549,1239430359,1205288764,2873335642,306140568,3126333856,3699539930,
            2339062667,4155918278,2144840660,129551188,3473357580,2613403541,3623004311,4102378124,4195837820,1704853223,3494605739,1289102284,790118878,3140362153,4074244823,2165367006,1463503520,1950189779,918791135,993369283,1745484251,2283179809,2103362647,3547909303,695986644,110026536,739667823,2385954605,270665668,489991280,4048372725,3736558612,3012062106,
            2018621034,1961692024,1988997307,1955843961,4061859667,3454845030,2413872493,348929559,3360622357,703116902,2977021943,2487377900,3130531837,3582467802,2945231790,3252620296,2015103974,2520537720,3174050848,3301510410,890415117,1324004857]]}}"#).unwrap();
        static ref TEST_CHALLENGE2: vdf::UnsolvedVDF = serde_json::from_str(r#"{"x":[1,[3116596062,3429917154,380242391,3128311776,375988634,2463438077,1261325488,1057941260]],"setup":{"t":[1,[1]],"N":[1,[3854412618,1010835532,4065591491,2102292353,92598763,3630267993,143952233,588018618,1202143563,3763776711,3919622616,3736236944,3019104952,1097139037,510520483,
            271524075,3750259967,338801097,3794457835,2616369307,2866577222,245019226,2857969932,2016285347,276111206,3518919836,1380023137,958480093,263236300,599239382,305388945,1684573828,3463971268,316587571,1308623964,3691975973,2110410231,20498320,3356443829,674970788,3158083955,2646109807,3973618680,1238793822,1613530525,2983843458,4198294090,2887288985,2305795058,
            1751043043,2360218609,675115021,2501880185,1137358181,1494832832,2977761473,1333077743,3908083095,3619922994,2477774598,1851774614,1986803699,654430673,2707032804,119999426,498239492,3923952010,960922580,3428006508,3717810843,819867535,802712456,3136895363,4206124604,392998340,3857199510,600699560,2956093857,4246036936,643980699,3054689974,3960330879,3022125176,
            1943348789,3511717571,951114303,4292692076,1563420755,2429423300,753953050,4244039215,3048110674,3107149417,3949931034,1819737890,2960219730,3228815506,1153460208,1768140778,2477772898,4115217101,234882067,2038431153,2965796120,1258007420,2929630642,2716201379,1549162426,2990350555,253519902,3056441647,275891275,3919792223,1398616677,2520384442,2301934163,2404379140,
            3626727849,1786031677,3946512759,1658684937,1602436348,1007504693,376286172,3276048846,3746742898,2658351446,70837396]]}}"#).unwrap();
    }

    #[tokio::test]
    async fn create_new_store() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = SledDB::open(dir.path().join("store")).await?;
        let _counter = store.increase_ping_counter().await?;
        dir.close()?;
        Ok(())
    }

    #[tokio::test]
    async fn open_existing_store() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = SledDB::open(dir.path().join("store")).await?;
        let counter_expected = store.increase_ping_counter().await?;
        drop(store);

        let store = SledDB::open(dir.path().join("store")).await?;
        let counter_actual = store.get_ping_counter().await?;
        assert_eq!(counter_expected, counter_actual);

        dir.close()?;
        Ok(())
    }

    async fn open_store() -> Result<(SledDB, tempfile::TempDir)> {
        let dir = tempfile::tempdir()?;
        let store = SledDB::open(dir.path().join("store")).await?;
        Ok((store, dir))
    }

    #[tokio::test]
    async fn increase_ping_counter() -> Result<()> {
        let (store, _guard) = open_store().await?;

        for counter_expected in 1..=10 {
            let counter_actual = store.increase_ping_counter().await?;
            assert_eq!(counter_expected, counter_actual);
            let another_counter_actual = store.get_ping_counter().await?;
            assert_eq!(counter_expected, another_counter_actual);
        }

        Ok(())
    }

    #[tokio::test]
    async fn set_challenge() -> Result<()> {
        let (store, _guard) = open_store().await?;

        let challenge = Challenge {
            id: 0,
            challenge: TEST_CHALLENGE.clone(),
        };
        store.set_challenge(challenge.clone()).await?;

        let stored_challenge = store.get_challenge().await?;
        assert_eq!(Some(challenge), stored_challenge);

        Ok(())
    }

    #[tokio::test]
    async fn not_allow_set_challenge_twice() -> Result<()> {
        let (store, _guard) = open_store().await?;

        let challenge1 = Challenge {
            id: 0,
            challenge: TEST_CHALLENGE.clone(),
        };
        store.set_challenge(challenge1.clone()).await?;

        let challenge2 = Challenge {
            id: 0,
            challenge: TEST_CHALLENGE2.clone(),
        };
        let result = store.set_challenge(challenge2.clone()).await;
        if let Err(SetChallengeError::AlreadySet(actual_challenge)) = result {
            assert_eq!(challenge1, actual_challenge);
        } else {
            panic!(
                "expected SetChallengeError::AlreadyExist error, got {:?}",
                result
            )
        }

        Ok(())
    }

    #[tokio::test]
    async fn erase_challenge_after_increasing_ping_counter() -> Result<()> {
        let (store, _guard) = open_store().await?;

        let challenge1 = Challenge {
            id: 0,
            challenge: TEST_CHALLENGE.clone(),
        };
        store.set_challenge(challenge1.clone()).await?;

        store.increase_ping_counter().await?;

        let stored_challenge = store.get_challenge().await?;
        assert_eq!(stored_challenge, None);

        Ok(())
    }

    #[tokio::test]
    async fn allow_setting_challenge_again_after_increasing_ping_counter() -> Result<()> {
        let (store, _guard) = open_store().await?;

        let challenge1 = Challenge {
            id: 0,
            challenge: TEST_CHALLENGE.clone(),
        };
        store.set_challenge(challenge1.clone()).await?;

        store.increase_ping_counter().await?;

        let challenge2 = Challenge {
            id: 1,
            challenge: TEST_CHALLENGE2.clone(),
        };
        store.set_challenge(challenge2.clone()).await?;

        let actual_challenge = store.get_challenge().await?;
        assert_eq!(actual_challenge, Some(challenge2));

        Ok(())
    }

    #[tokio::test]
    async fn remember_server_secret_share() -> Result<()> {
        let (store, _guard) = open_store().await?;

        let pk = &b"dummy public key"[..];
        let sk = &b"dummy secret share"[..];

        store.add_server_secret_share(pk, sk).await?;

        let actual_sk = store.get_server_secret_share(pk).await?;
        assert_eq!(
            Some(sk),
            actual_sk.as_ref().map(|sk| sk.secret_share().as_ref())
        );

        Ok(())
    }

    #[tokio::test]
    async fn return_none_if_server_share_not_found() -> Result<()> {
        let (store, _guard) = open_store().await?;

        let pk = &b"dummy public key"[..];
        let actual_sk = store.get_server_secret_share(pk).await?;
        assert!(actual_sk.is_none());

        Ok(())
    }
}
