use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use chrono::Utc;
use colored::*;
use std::fmt;

// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: f64,
    timestamp: i64,
}

impl Transaction {
    fn new(sender: String, receiver: String, amount: f64) -> Self {
        Self {
            sender,
            receiver,
            amount,
            timestamp: Utc::now().timestamp(),
        }
    }

    fn to_string(&self) -> String {
        format!("{}{}{}{}", self.sender, self.receiver, self.amount, self.timestamp)
    }
}

// Block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: i64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    hash: String,
    nonce: u64,
    difficulty: usize,
}

impl Block {
    fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String, difficulty: usize) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Self {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            difficulty,
        };
        block.hash = block.calculate_hash();
        block
    }

    fn calculate_hash(&self) -> String {
        let transactions_str: String = self.transactions
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join("");

        let block_data = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, transactions_str, self.previous_hash, self.nonce
        );

        let mut hasher = Sha256::new();
        hasher.update(block_data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn mine_block(&mut self) {
        let target = "0".repeat(self.difficulty);
        
        println!("\n{}", "⛏️  Mining block...".bright_yellow().bold());
        print!("Nonce: ");

        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
            
            // Show progress every 10000 attempts
            if self.nonce % 10000 == 0 {
                print!("{} ", self.nonce.to_string().bright_cyan());
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }

        println!("\n{} Block mined! Hash: {}", 
            "✓".bright_green().bold(), 
            self.hash.bright_green()
        );
        println!("Nonce found: {}", self.nonce.to_string().bright_cyan().bold());
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n{}\n", "═".repeat(80).bright_blue())?;
        write!(f, "{} {}\n", "Block #".bright_white().bold(), self.index.to_string().bright_cyan().bold())?;
        write!(f, "{}\n", "─".repeat(80).bright_blue())?;
        write!(f, "{}: {}\n", "Timestamp".bright_white(), self.timestamp)?;
        write!(f, "{}: {}\n", "Previous Hash".bright_white(), self.previous_hash.bright_yellow())?;
        write!(f, "{}: {}\n", "Hash".bright_white(), self.hash.bright_green())?;
        write!(f, "{}: {}\n", "Nonce".bright_white(), self.nonce.to_string().bright_cyan())?;
        write!(f, "{}: {}\n", "Difficulty".bright_white(), self.difficulty)?;
        write!(f, "\n{}\n", "Transactions:".bright_white().bold())?;
        
        for (i, tx) in self.transactions.iter().enumerate() {
            write!(f, "  {}. {} {} → {} {} coins\n",
                i + 1,
                tx.sender.bright_magenta(),
                "→".bright_white(),
                tx.receiver.bright_magenta(),
                tx.amount.to_string().bright_yellow()
            )?;
        }
        write!(f, "{}\n", "═".repeat(80).bright_blue())
    }
}

// Blockchain structure
struct Blockchain {
    chain: Vec<Block>,
    difficulty: usize,
    pending_transactions: Vec<Transaction>,
    mining_reward: f64,
}

impl Blockchain {
    fn new(difficulty: usize, mining_reward: f64) -> Self {
        let mut blockchain = Self {
            chain: Vec::new(),
            difficulty,
            pending_transactions: Vec::new(),
            mining_reward,
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_tx = Transaction::new(
            "System".to_string(),
            "Genesis".to_string(),
            0.0,
        );
        let mut genesis_block = Block::new(0, vec![genesis_tx], "0".to_string(), self.difficulty);
        genesis_block.mine_block();
        self.chain.push(genesis_block);
    }

    fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
        println!("{} Transaction added to pending pool", "✓".bright_green().bold());
    }

    fn mine_pending_transactions(&mut self, miner_address: String) {
        // Add mining reward transaction
        let reward_tx = Transaction::new(
            "System".to_string(),
            miner_address.clone(),
            self.mining_reward,
        );
        self.pending_transactions.push(reward_tx);

        let previous_hash = self.get_latest_block().hash.clone();
        let index = self.chain.len() as u64;
        
        let mut new_block = Block::new(
            index,
            self.pending_transactions.clone(),
            previous_hash,
            self.difficulty,
        );

        new_block.mine_block();
        self.chain.push(new_block);

        self.pending_transactions = Vec::new();
    }

    fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            // Verify hash is correct
            if current_block.hash != current_block.calculate_hash() {
                println!("{} Block #{} has invalid hash!", "✗".bright_red().bold(), i);
                return false;
            }

            // Verify chain linkage
            if current_block.previous_hash != previous_block.hash {
                println!("{} Block #{} has invalid previous hash!", "✗".bright_red().bold(), i);
                return false;
            }

            // Verify proof of work
            let target = "0".repeat(current_block.difficulty);
            if !current_block.hash.starts_with(&target) {
                println!("{} Block #{} has invalid proof of work!", "✗".bright_red().bold(), i);
                return false;
            }
        }

        true
    }

    fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;

        for block in &self.chain {
            for tx in &block.transactions {
                if tx.sender == address {
                    balance -= tx.amount;
                }
                if tx.receiver == address {
                    balance += tx.amount;
                }
            }
        }

        balance
    }

    fn display(&self) {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════════════════════════╗".bright_blue().bold());
        println!("{}", "║                           🔗 RUSTY BLOCKCHAIN 🔗                             ║".bright_blue().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════════════════════════╝".bright_blue().bold());
        
        for block in &self.chain {
            print!("{}", block);
        }
    }
}

fn main() {
    println!("\n{}", "🚀 Welcome to Rusty Blockchain! 🚀".bright_cyan().bold());
    println!("{}\n", "Building a blockchain from scratch...".bright_white());

    // Create blockchain with difficulty 4 and mining reward of 100 coins
    let mut blockchain = Blockchain::new(4, 100.0);

    println!("\n{}", "📝 Adding transactions...".bright_yellow().bold());
    
    // Add some transactions
    blockchain.add_transaction(Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        50.0,
    ));

    blockchain.add_transaction(Transaction::new(
        "Bob".to_string(),
        "Charlie".to_string(),
        25.0,
    ));

    // Mine block 1
    println!("\n{}", "⛏️  Mining Block #1...".bright_yellow().bold());
    blockchain.mine_pending_transactions("Miner1".to_string());

    // Add more transactions
    println!("\n{}", "📝 Adding more transactions...".bright_yellow().bold());
    blockchain.add_transaction(Transaction::new(
        "Charlie".to_string(),
        "Alice".to_string(),
        10.0,
    ));

    blockchain.add_transaction(Transaction::new(
        "Alice".to_string(),
        "Miner1".to_string(),
        5.0,
    ));

    // Mine block 2
    println!("\n{}", "⛏️  Mining Block #2...".bright_yellow().bold());
    blockchain.mine_pending_transactions("Miner1".to_string());

    // Display the entire blockchain
    blockchain.display();

    // Check balances
    println!("\n{}", "💰 Account Balances:".bright_yellow().bold());
    println!("{}\n", "─".repeat(50).bright_blue());
    
    let addresses = vec!["Alice", "Bob", "Charlie", "Miner1"];
    for address in addresses {
        let balance = blockchain.get_balance(address);
        println!("{}: {} coins", 
            address.bright_magenta().bold(), 
            balance.to_string().bright_green()
        );
    }

    // Validate blockchain
    println!("\n{}", "🔍 Validating blockchain...".bright_yellow().bold());
    if blockchain.is_chain_valid() {
        println!("{} Blockchain is valid!", "✓".bright_green().bold());
    } else {
        println!("{} Blockchain is invalid!", "✗".bright_red().bold());
    }

    // Demonstrate tampering detection
    println!("\n{}", "🔓 Attempting to tamper with blockchain...".bright_red().bold());
    if blockchain.chain.len() > 1 {
        blockchain.chain[1].transactions[0].amount = 1000.0;
        println!("Changed transaction amount in Block #1");
        
        println!("\n{}", "🔍 Re-validating blockchain...".bright_yellow().bold());
        if blockchain.is_chain_valid() {
            println!("{} Blockchain is still valid!", "✓".bright_green().bold());
        } else {
            println!("{} Tampering detected! Blockchain is now invalid!", "✗".bright_red().bold());
        }
    }

    println!("\n{}", "🎉 Demo complete!".bright_cyan().bold());
}