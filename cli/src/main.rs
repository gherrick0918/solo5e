use clap::{Parser, ValueEnum};
use engine::{AdMode, Dice};

#[derive(Copy, Clone, ValueEnum)]
enum Adv {
    Normal,
    Advantage,
    Disadvantage,
}

#[derive(Parser)]
struct Args {
    /// RNG seed for determinism
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Advantage mode
    #[arg(long, value_enum, default_value_t = Adv::Normal)]
    adv: Adv,
    /// Number of rolls
    #[arg(long, default_value_t = 5)]
    rolls: u32,
}

fn main() {
    let args = Args::parse();
    let mode = match args.adv {
        Adv::Normal => AdMode::Normal,
        Adv::Advantage => AdMode::Advantage,
        Adv::Disadvantage => AdMode::Disadvantage,
    };
    let mut dice = Dice::from_seed(args.seed);
    for _ in 0..args.rolls {
        println!("{}", dice.d20(mode));
    }
}
