use anchor_lang::error_code;

#[error_code]
pub enum TradeError {
    #[msg("Round: Not allowed owner")]
    NotAllowedOwner,
}