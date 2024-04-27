use anchor_lang::error_code;

#[error_code]
pub enum TradeError {
    #[msg("Trade: Not allowed owner")]
    NotAllowedOwner,

    #[msg("Trade: This is invalid token address")]
    InvalidTokenAddress,

    #[msg("Trade: Please deposit USDC more than 0")]
    ZeroAmount
}