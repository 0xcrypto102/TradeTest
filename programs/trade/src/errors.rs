use anchor_lang::error_code;

#[error_code]
pub enum TradeError {
    #[msg("Trade: Not allowed owner")]
    NotAllowedOwner,

    #[msg("Trade: This is invalid token address")]
    InvalidTokenAddress,

    #[msg("Trade: Please deposit USDC more than 0")]
    ZeroAmount,

    #[msg("Trade: This pubkey is invalid price feed")]
    InvalidPriceFeed,

    #[msg("Trade: There isn't balance in vault")]
    NoBalance,

    #[msg("Trade: There are insufficient fund in vault")]
    Insufficientfund,
}