use std::convert::TryInto;

use casper_types::{
    bytesrepr::{self, FromBytes},
    ApiError, CLType, CLTyped,
};

use core::convert::TryFrom;
/// error for flashlender
#[derive(Debug)]
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
    FlashLenderOverflow,
}

// 65535  u16::MAX
///65335
const ERROR_FLASH_MINTER_UNSUPPORTED_CURRENCY: u16 = u16::MAX - 200;
///65334
const ERROR_FLASH_MINTER_CALLBACK_FAILED: u16 = u16::MAX - 201;
///65333
const ERROR_FLASH_MINTER_REPAY_NOT_APPROVED: u16 = u16::MAX - 202;
///65332
const ERROR_FLASH_LENDER_UNSUPPORTED_CURRENCY: u16 = u16::MAX - 203;
///65331
const ERROR_FLASH_LENDER_TRANSFER_FAILED: u16 = u16::MAX - 204;
///65330
const ERROR_FLASH_LENDER_CALLBACK_FAILED: u16 = u16::MAX - 205;
///65329
const ERROR_FLASH_LENDER_REPAY_FAILED: u16 = u16::MAX - 206;
///65328
const ERROR_IERC3156_CALLBACK_FAILED: u16 = u16::MAX - 207;
///65327
const ERROR_FLASH_BORROWER_UNTRUSTED_FENDER: u16 = u16::MAX - 208;
///65535 - 209 = 65326
const ERROR_FFLASH_BORROWER_UNTRUSTED_LOAN_INITIATOR: u16 = u16::MAX - 209;
///65535 - 210 = 65325
const ERROR_FFLASH_LENDER_OVERFLOW: u16 = u16::MAX - 210;

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
            Error::FlashLenderOverflow => ERROR_FFLASH_LENDER_OVERFLOW,
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
