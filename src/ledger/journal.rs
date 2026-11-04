use crate::domain::Paise;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Represents a single leg of a transaction (either a Debit or a Credit to an account).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLeg {
    pub account: String,
    pub amount: Paise,
    pub is_debit: bool, // true = Debit (Left side), false = Credit (Right side)
}

/// Represents an immutable, audit-ready journal transaction containing one or more legs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub legs: Vec<TransactionLeg>,
    pub description: String,
    pub previous_hash: String,
    pub entry_hash: String,
}

impl JournalEntry {
    /// Creates a multi-leg journal entry after strictly verifying that 
    /// Total Debits == Total Credits using our loop and if/else accumulator.
    pub fn create(
        legs: Vec<TransactionLeg>,
        description: impl Into<String>,
        timestamp: DateTime<Utc>,
        previous_hash: impl Into<String>,
    ) -> Result<Self, String> {
        let description = description.into();
        let previous_hash = previous_hash.into();

        // 1. Validate that the transaction has legs
        if legs.is_empty() {
            return Err("A journal entry must have at least one transaction leg.".to_string());
        }

        // 2. Accumulate debits and credits using the loop and if/else logic
        let mut total_debits = 0i64;
        let mut total_credits = 0i64;

        for leg in &legs {
            if leg.amount.0 <= 0 {
                return Err(format!("Leg amount for account '{}' must be greater than zero.", leg.account));
            }

            // The if/else sorting mechanism separating left (debit) and right (credit) buckets
            if leg.is_debit {
                total_debits = total_debits.checked_add(leg.amount.0)
                    .ok_or("Integer overflow calculating total debits")?;
            } else {
                total_credits = total_credits.checked_add(leg.amount.0)
                    .ok_or("Integer overflow calculating total credits")?;
            }
        }

        // 3. THE GOLDEN RULE CHECK: Compare both buckets after the loop finishes
        if total_debits != total_credits {
            return Err(format!(
                "Transaction out of balance! Total Debits: ₹{}, Total Credits: ₹{}",
                Paise(total_debits), Paise(total_credits)
            ));
        }

        let id = Uuid::v4();

        // 4. Compute SHA-256 cryptographic hash for tamper-evident audit chaining
        let entry_hash = Self::calculate_hash(&id, &timestamp, &legs, &description, &previous_hash);

        Ok(Self {
            id,
            timestamp,
            legs,
            description,
            previous_hash,
            entry_hash,
        })
    }

    /// Helper to compute the cryptographic hash of the entry.
    fn calculate_hash(
        id: &Uuid,
        timestamp: &DateTime<Utc>,
        legs: &[TransactionLeg],
        desc: &str,
        prev_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.update(timestamp.to_string().as_bytes());
        for leg in legs {
            hasher.update(leg.account.as_bytes());
            hasher.update(leg.amount.0.to_be_bytes());
            hasher.update(&[if leg.is_debit { 1 } else { 0 }]);
        }
        hasher.update(desc.as_bytes());
        hasher.update(prev_hash.as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Verifies if the entry's hash is valid and untampered.
    pub fn verify_integrity(&self) -> bool {
        let expected_hash = Self::calculate_hash(
            &self.id,
            &self.timestamp,
            &self.legs,
            &self.description,
            &self.previous_hash,
        );
        self.entry_hash == expected_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced_transaction() {
        let now = Utc::now();
        let genesis_hash = "0000000000000000000000000000000000000000000000000000000000000000";

        // User deposits ₹1,000 via UPI (1 Debit, 1 Credit)
        let legs = vec![
            TransactionLeg {
                account: "asset:bank_axis".to_string(),
                amount: Paise::from_rupees(1000),
                is_debit: true, 
            },
            TransactionLeg {
                account: "liability:user_wallet_101".to_string(),
                amount: Paise::from_rupees(1000),
                is_debit: false, 
            },
        ];

        let entry = JournalEntry::create(legs, "Wallet top-up", now, genesis_hash);
        assert!(entry.is_ok());
        assert!(entry.unwrap().verify_integrity());
    }

    #[test]
    fn test_unbalanced_transaction_rejected() {
        let now = Utc::now();
        let genesis_hash = "0000000000000000000000000000000000000000000000000000000000000000";

        // Malformed transaction: Debit ₹1,000 but Credit only ₹900 (Should fail check)
        let legs = vec![
            TransactionLeg {
                account: "asset:bank_axis".to_string(),
                amount: Paise::from_rupees(1000),
                is_debit: true,
            },
            TransactionLeg {
                account: "liability:user_wallet_101".to_string(),
                amount: Paise::from_rupees(900), 
                is_debit: false,
            },
        ];

        let entry = JournalEntry::create(legs, "Broken transaction", now, genesis_hash);
        assert!(entry.is_err());
    }
}