use bytemuck::Contiguous;
use solana_program::program_error::ProgramError;

use num_enum::IntoPrimitive;
use thiserror::Error;

pub type MangoResult<T = ()> = Result<T, EntropyError>;

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum SourceFileId {
    Processor = 0,
    State = 1,
    Critbit = 2,
    Queue = 3,
    Matching = 4,
    Oracle = 5,
}

impl std::fmt::Display for SourceFileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceFileId::Processor => write!(f, "src/processor.rs"),
            SourceFileId::State => write!(f, "src/state.rs"),
            SourceFileId::Critbit => write!(f, "src/critbit"),
            SourceFileId::Queue => write!(f, "src/queue.rs"),
            SourceFileId::Matching => write!(f, "src/matching.rs"),
            SourceFileId::Oracle => write!(f, "src/oracle.rs"),
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum EntropyError {
    #[error(transparent)]
    ProgramError(#[from] ProgramError),
    #[error("{mango_error_code}; {source_file_id}:{line}")]
    EntropyErrorCode { mango_error_code: EntropyErrorCode, line: u32, source_file_id: SourceFileId },
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum EntropyErrorCode {
    #[error("EntropyErrorCode::InvalidCache")]
    InvalidCache,
    #[error("EntropyErrorCode::InvalidOwner")]
    InvalidOwner,
    #[error("EntropyErrorCode::InvalidGroupOwner")]
    InvalidGroupOwner,
    #[error("EntropyErrorCode::InvalidSignerKey")]
    InvalidSignerKey,
    #[error("EntropyErrorCode::InvalidAdminKey")]
    InvalidAdminKey,
    #[error("EntropyErrorCode::InvalidVault")]
    InvalidVault,
    #[error("EntropyErrorCode::MathError")]
    MathError,
    #[error("EntropyErrorCode::InsufficientFunds")]
    InsufficientFunds,
    #[error("EntropyErrorCode::InvalidToken")]
    InvalidToken,
    #[error("EntropyErrorCode::InvalidMarket")]
    InvalidMarket,
    #[error("EntropyErrorCode::InvalidProgramId")]
    InvalidProgramId,
    #[error("EntropyErrorCode::GroupNotRentExempt")]
    GroupNotRentExempt,
    #[error("EntropyErrorCode::OutOfSpace")]
    OutOfSpace,
    #[error("EntropyErrorCode::TooManyOpenOrders Reached the maximum number of open orders for this market")]
    TooManyOpenOrders,

    #[error("EntropyErrorCode::AccountNotRentExempt")]
    AccountNotRentExempt,

    #[error("EntropyErrorCode::ClientIdNotFound")]
    ClientIdNotFound,
    #[error("EntropyErrorCode::InvalidNodeBank")]
    InvalidNodeBank,
    #[error("EntropyErrorCode::InvalidRootBank")]
    InvalidRootBank,
    #[error("EntropyErrorCode::MarginBasketFull")]
    MarginBasketFull,
    #[error("EntropyErrorCode::NotLiquidatable")]
    NotLiquidatable,
    #[error("EntropyErrorCode::Unimplemented")]
    Unimplemented,
    #[error("EntropyErrorCode::PostOnly")]
    PostOnly,
    #[error("EntropyErrorCode::Bankrupt Invalid instruction for bankrupt account")]
    Bankrupt,
    #[error("EntropyErrorCode::InsufficientHealth")]
    InsufficientHealth,
    #[error("EntropyErrorCode::InvalidParam")]
    InvalidParam,
    #[error("EntropyErrorCode::InvalidAccount")]
    InvalidAccount,
    #[error("EntropyErrorCode::InvalidAccountState")]
    InvalidAccountState,
    #[error("EntropyErrorCode::SignerNecessary")]
    SignerNecessary,
    #[error("EntropyErrorCode::InsufficientLiquidity Not enough deposits in this node bank")]
    InsufficientLiquidity,
    #[error("EntropyErrorCode::InvalidOrderId")]
    InvalidOrderId,
    #[error("EntropyErrorCode::InvalidOpenOrdersAccount")]
    InvalidOpenOrdersAccount,
    #[error("EntropyErrorCode::BeingLiquidated Invalid instruction while being liquidated")]
    BeingLiquidated,
    #[error("EntropyErrorCode::InvalidRootBankCache Cache the root bank to resolve")]
    InvalidRootBankCache,
    #[error("EntropyErrorCode::InvalidPriceCache Cache the oracle price to resolve")]
    InvalidPriceCache,
    #[error("EntropyErrorCode::InvalidPerpMarketCache Cache the perp market to resolve")]
    InvalidPerpMarketCache,
    #[error("EntropyErrorCode::TriggerConditionFalse The trigger condition for this TriggerOrder is not met")]
    TriggerConditionFalse,
    #[error("EntropyErrorCode::InvalidSeeds Invalid seeds. Unable to create PDA")]
    InvalidSeeds,
    #[error("EntropyErrorCode::InvalidOracleType The oracle account was not recognized")]
    InvalidOracleType,
    #[error("EntropyErrorCode::InvalidOraclePrice")]
    InvalidOraclePrice,
    #[error("invalid serum fees vault")]
    InvalidSerumVault,
    #[error("EntropyErrorCode::Default Check the source code for more info")]
    Default = u32::MAX_VALUE,
}

impl From<EntropyError> for ProgramError {
    fn from(e: EntropyError) -> ProgramError {
        match e {
            EntropyError::ProgramError(pe) => pe,
            EntropyError::EntropyErrorCode { mango_error_code, line: _, source_file_id: _ } => {
                ProgramError::Custom(mango_error_code.into())
            }
        }
    }
}

impl From<serum_dex::error::DexError> for EntropyError {
    fn from(de: serum_dex::error::DexError) -> Self {
        let pe: ProgramError = de.into();
        pe.into()
    }
}

#[inline]
pub fn check_assert(
    cond: bool,
    mango_error_code: EntropyErrorCode,
    line: u32,
    source_file_id: SourceFileId,
) -> MangoResult<()> {
    if cond {
        Ok(())
    } else {
        Err(EntropyError::EntropyErrorCode { mango_error_code, line, source_file_id })
    }
}

#[macro_export]
macro_rules! declare_check_assert_macros {
    ($source_file_id:expr) => {
        #[allow(unused_macros)]
        macro_rules! check {
            ($cond:expr, $err:expr) => {
                check_assert($cond, $err, line!(), $source_file_id)
            };
        }

        #[allow(unused_macros)]
        macro_rules! check_eq {
            ($x:expr, $y:expr, $err:expr) => {
                check_assert($x == $y, $err, line!(), $source_file_id)
            };
        }

        #[allow(unused_macros)]
        macro_rules! throw {
            () => {
                EntropyError::EntropyErrorCode {
                    mango_error_code: EntropyErrorCode::Default,
                    line: line!(),
                    source_file_id: $source_file_id,
                }
            };
        }

        #[allow(unused_macros)]
        macro_rules! throw_err {
            ($err:expr) => {
                EntropyError::EntropyErrorCode {
                    mango_error_code: $err,
                    line: line!(),
                    source_file_id: $source_file_id,
                }
            };
        }

        #[allow(unused_macros)]
        macro_rules! math_err {
            () => {
                EntropyError::EntropyErrorCode {
                    mango_error_code: EntropyErrorCode::MathError,
                    line: line!(),
                    source_file_id: $source_file_id,
                }
            };
        }
    };
}
