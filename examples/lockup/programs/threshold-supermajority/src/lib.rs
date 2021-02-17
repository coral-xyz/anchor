//! An adjudicator function for the voting program. A propsal is considered
//! to have passed if *both* 1 billion votes (stake tokens) are in favor
//! and of those who voted, 60% of all votes say yes.

#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use voting::{Adjudicator, Empty, Proposal};

const THRESHOLD: u64 = 2_500_000_000;
const MAJORITY_PERCENT: u64 = 75;

#[program]
pub mod threshold_supermajority {
    use super::*;

    #[state]
    pub struct ThresholdSupermajority;

    impl Adjudicator for ThresholdSupermajority {
        fn did_vote_pass(_ctx: Context<Empty>, proposal: Proposal) -> ProgramResult {
            let total_votes = proposal.vote_yes + proposal.vote_no;

            if total_votes < THRESHOLD {
                return Err(ErrorCode::BelowThreshold.into());
            }

            // Adjust to avoid floating point.
            let adjusted = proposal.vote_yes * 100;
            if (adjusted / total_votes) < MAJORITY_PERCENT {
                return Err(ErrorCode::InvalidSupermajority.into());
            }

            Ok(())
        }
    }
}

#[error]
pub enum ErrorCode {
    BelowThreshold,
    InvalidSupermajority,
}
