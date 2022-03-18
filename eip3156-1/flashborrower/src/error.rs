use std::convert::TryInto;

use casper_types::{
    bytesrepr::{self, FromBytes},
    ApiError, CLType, CLTyped,
};

use core::convert::TryFrom;

pub enum Error {
    FlashMinterUnsupportedCurrency,
    FlashMinterCallbackFailed,
    FlashMinterRepayNotApproved,
    FlashLenderUnsupportedCurrency,
    FlashLenderTransferFailed,
    FlashLenderCallbackFailed,
    FlashLenderRepayFailed,
    IERC3156CallbackFailed,
    FlashBorrowerUntrustedFender,
    FlashBorrowerUntrustedLoanInitiator,
    FlashBorrowerInitialFailed,
}

///65435
const ERROR_FLASH_MINTER_UNSUPPORTED_CURRENCY: u16 = u16::MAX - 100;
///65434
const ERROR_FLASH_MINTER_CALLBACK_FAILED: u16 = u16::MAX - 101;
///65433
const ERROR_FLASH_MINTER_REPAY_NOT_APPROVED: u16 = u16::MAX - 102;
///65432
const ERROR_FLASH_LENDER_UNSUPPORTED_CURRENCY: u16 = u16::MAX - 103;
///65431
const ERROR_FLASH_LENDER_TRANSFER_FAILED: u16 = u16::MAX - 104;
///65430
const ERROR_FLASH_LENDER_CALLBACK_FAILED: u16 = u16::MAX - 105;
///65429
const ERROR_FLASH_LENDER_REPAY_FAILED: u16 = u16::MAX - 106;
///65428
const ERROR_IERC3156_CALLBACK_FAILED: u16 = u16::MAX - 107;
///65427
const ERROR_FLASH_BORROWER_UNTRUSTED_FENDER: u16 = u16::MAX - 108;
///65535 - 109 = 65426
const ERROR_FFLASH_BORROWER_UNTRUSTED_LOAN_INITIATOR: u16 = u16::MAX - 109;
///65535 - 110 = 65425
const ERROR_FLASH_BORROWER_INITAL_FAILED: u16 = u16::MAX - 110;

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        let user_error = match error {
            Error::FlashMinterUnsupportedCurrency => ERROR_FLASH_MINTER_UNSUPPORTED_CURRENCY,
            Error::FlashMinterCallbackFailed => ERROR_FLASH_MINTER_CALLBACK_FAILED,
            Error::FlashMinterRepayNotApproved => ERROR_FLASH_MINTER_REPAY_NOT_APPROVED,
            Error::FlashLenderUnsupportedCurrency => ERROR_FLASH_LENDER_UNSUPPORTED_CURRENCY,
            Error::FlashLenderTransferFailed => ERROR_FLASH_LENDER_TRANSFER_FAILED,
            Error::FlashLenderCallbackFailed => ERROR_FLASH_LENDER_CALLBACK_FAILED,
            Error::FlashLenderRepayFailed => ERROR_FLASH_LENDER_REPAY_FAILED,
            Error::IERC3156CallbackFailed => ERROR_IERC3156_CALLBACK_FAILED,
            Error::FlashBorrowerUntrustedFender => ERROR_FLASH_BORROWER_UNTRUSTED_FENDER,
            Error::FlashBorrowerUntrustedLoanInitiator => {
                ERROR_FFLASH_BORROWER_UNTRUSTED_LOAN_INITIATOR
            }
            Error::FlashBorrowerInitialFailed => ERROR_FLASH_BORROWER_INITAL_FAILED,
        };
        ApiError::User(user_error)
    }
}

pub struct TryFromU8ForError(());

impl CLTyped for Error {
    fn cl_type() -> CLType {
        CLType::U8
    }
}

impl TryFrom<u8> for Error {
    type Error = TryFromU8ForError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            d if d == Error::FlashMinterUnsupportedCurrency as u8 => {
                Ok(Error::FlashMinterUnsupportedCurrency)
            }
            d if d == Error::FlashMinterCallbackFailed as u8 => {
                Ok(Error::FlashMinterCallbackFailed)
            }
            d if d == Error::FlashMinterRepayNotApproved as u8 => {
                Ok(Error::FlashMinterRepayNotApproved)
            }
            d if d == Error::FlashLenderUnsupportedCurrency as u8 => {
                Ok(Error::FlashLenderUnsupportedCurrency)
            }
            d if d == Error::FlashLenderTransferFailed as u8 => {
                Ok(Error::FlashLenderTransferFailed)
            }
            d if d == Error::FlashLenderCallbackFailed as u8 => {
                Ok(Error::FlashLenderCallbackFailed)
            }
            d if d == Error::FlashLenderRepayFailed as u8 => Ok(Error::FlashLenderRepayFailed),
            d if d == Error::IERC3156CallbackFailed as u8 => Ok(Error::IERC3156CallbackFailed),
            d if d == Error::FlashBorrowerUntrustedFender as u8 => {
                Ok(Error::FlashBorrowerUntrustedFender)
            }
            d if d == Error::FlashBorrowerUntrustedFender as u8 => {
                Ok(Error::FlashBorrowerUntrustedFender)
            }
            d if d == Error::FlashBorrowerUntrustedLoanInitiator as u8 => {
                Ok(Error::FlashBorrowerUntrustedLoanInitiator)
            }
            _ => Err(TryFromU8ForError(())),
        }
    }
}

impl FromBytes for Error {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (value, rem): (u8, _) = FromBytes::from_bytes(bytes)?;
        let error: Error = value
            .try_into()
            // In case an Error variant is unable to be determined it would return an
            // Error::Formatting as if its unable to be correctly deserialized.
            .map_err(|_| bytesrepr::Error::Formatting)?;
        Ok((error, rem))
    }
}
