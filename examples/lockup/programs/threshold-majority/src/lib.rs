//! An adjudicator function for the voting program. A propsal is considered
//! to have passed if *both* 2.5 billion votes (stake tokens) are in favor
//! and of those who voted, 75% of all votes say yes.

#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use voting::{Adjudicator, Empty, Proposal};

const THRESHOLD: u64 = 1_000_000_000;
const MAJORITY_PERCENT: u64 = 60;

#[program]
pub mod threshold_majority {
    use super::*;

    #[state]
    pub struct ThresholdMajority;

    impl Adjudicator for ThresholdMajority {
        fn did_vote_pass(_ctx: Context<Empty>, proposal: Proposal) -> ProgramResult {
            let total_votes = proposal.vote_yes + proposal.vote_no;

            if total_votes < THRESHOLD {
                return Err(ErrorCode::BelowThreshold.into());
            }

            // Adjust to avoid floating point.
            let adjusted = proposal.vote_yes * 100;
            if (adjusted / total_votes) < MAJORITY_PERCENT {
                return Err(ErrorCode::InvalidMajority.into());
            }

            Ok(())
        }
    }
}

#[error]
pub enum ErrorCode {
    BelowThreshold,
    InvalidMajority,
}
