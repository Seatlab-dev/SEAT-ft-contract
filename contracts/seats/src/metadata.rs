use crate::Seats;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_sdk::{json_types::Base64VecU8, near_bindgen};

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

/// The human-readable name of the token.
pub const NAME: &str = "SeatlabNFT";
/// The abbreviation, like wETH or AMPL.
pub const SYMBOL: &str = "SEAT";
/// A small image associated with this token.
/// Must be a data URL, to help consumers display it quickly while protecting user
/// data.
///
/// In this case, this is a URL-escaped version of:
/// `data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg'><rect width='50' height='50' /></svg>`.
///
pub const ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' version='1.1' x='0px' y='0px' width='32px' height='32px' viewBox='0 0 32 32' style='enable-background:new 0 0 32 32;' xml:space='preserve'%3E%3Cstyle type='text/css'%3E .st0%7Bfill:%23050035;%7D .st1%7Bfill:url(%23SVGID_1_);%7D%0A%3C/style%3E%3Cg id='Layer_1'%3E%3Ccircle class='st0' cx='16' cy='16' r='16'/%3E%3C/g%3E%3Cg id='Isolation_Mode'%3E%3ClinearGradient id='SVGID_1_' gradientUnits='userSpaceOnUse' x1='6.2375' y1='16' x2='25.7625' y2='16'%3E%3Cstop offset='0' style='stop-color:%2300C2FF'/%3E%3Cstop offset='1' style='stop-color:%230067FF'/%3E%3C/linearGradient%3E%3Cpath class='st1' d='M14,6.1c-0.3,0-0.5,0.1-0.8,0.2L8.1,9.4L7.8,9.6L7.2,10c-0.5,0.3-0.9,0.6-0.9,1.7v4.1c0,0.9,0.3,1.6,0.9,1.9 l6.2,3.8c1,0.6,2.3-0.1,2.3-1.3v-2.9c0-0.7-0.4-1.4-1-1.8l-2.9-1.8l3-1.9c0.5-0.3,0.8-0.9,0.8-1.5V7.6C15.6,6.7,14.8,6.1,14,6.1z M14.7,10.4c0,0.3-0.2,0.6-0.4,0.8l-3.4,2.1l-1.8-1.1c-0.5-0.3-0.8-0.5-1.1-0.8c-0.3-0.3-0.2-0.8,0.2-1c0,0,0,0,0,0L13.7,7 C13.8,7,13.9,6.9,14,6.9c0.3,0,0.7,0.3,0.7,0.7V10.4z M24.9,14.3l-6.2-3.8c-1-0.6-2.3,0.1-2.3,1.3v2.9c0,0.7,0.4,1.4,1,1.8l2.9,1.8 l-3,1.9c-0.5,0.3-0.8,0.9-0.8,1.5v2.8c0,0.9,0.8,1.6,1.6,1.6c0.3,0,0.5-0.1,0.8-0.2l5.1-3.1l0.4-0.2l0.6-0.4 c0.5-0.3,0.9-0.6,0.9-1.7v-4.1C25.8,15.3,25.4,14.6,24.9,14.3z M23.8,21.6C23.8,21.6,23.8,21.6,23.8,21.6L18.3,25 c-0.1,0.1-0.2,0.1-0.4,0.1c-0.3,0-0.7-0.3-0.7-0.7v-2.8c0-0.3,0.2-0.6,0.4-0.8l3.4-2.1l1.8,1.1c0.5,0.3,0.8,0.5,1.1,0.8 C24.3,20.9,24.2,21.4,23.8,21.6z'/%3E%3C/g%3E%3C/svg%3E";
/// Used in frontends to show the proper significant digits of a token.
/// This concept is explained well in this
/// [OpenZeppelin post](https://docs.openzeppelin.com/contracts/3.x/erc20#a-note-on-decimals).
pub const DECIMALS: u8 = 5;
/// A link to a valid JSON file containing various keys offering supplementary
/// details on the token.
///
/// Examples:  
/// `/ipfs/QmdmQXB2mzChmMeKY47C43LxUdg1NDJ5MWcKMKxDu7RgQm`
/// `https://example.com/token.json`
///
/// If the information given in this document conflicts with the on-chain
/// attributes, the values in reference shall be considered the source of truth.
pub const REFERENCE: Option<String> = None;
/// The base64-encoded sha256 hash of the JSON file contained in the reference
/// field. This is to guard against off-chain tampering.
pub const REFERENCE_HASH: Option<Base64VecU8> = None;

/// Based on [`near_contract_standards::fungible_token::metadata::FungibleTokenMetadata::assert_valid()`].
pub fn check(metadata: &FungibleTokenMetadata) {
    use near_contract_standards::fungible_token::metadata::FT_METADATA_SPEC;
    use near_sdk::require;

    require!(
        metadata.spec == FT_METADATA_SPEC,
        r#"metadata.spec expected to be `"ft-1.0.0"`"#
    );
    require!(metadata.reference.is_some() == metadata.reference_hash.is_some(), "if either metadata.reference or metadata.reference_hash is `null`, the other one must also be `null`");
    if let Some(reference_hash) = &metadata.reference_hash {
        require!(
            reference_hash.0.len() == 32,
            "metadata.reference_hash has to be 32 bytes"
        );
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Seats {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
