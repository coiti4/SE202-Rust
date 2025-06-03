use clap::Parser;

#[derive(Parser)]
#[command(about = "Compute Fibonacci suite values", long_about = None)]
struct CliArgs {
    /// The maximal number to print the fibo value of
    value: u32,

    /// Print intermediate values
    #[arg(short, long)]
    verbose: bool,

    /// The minimum number to compute
    #[arg(short, long, value_name = "NUMBER")]
    min: Option<u32>,
}
    
fn main() {
    let args = CliArgs::parse();

    let value = args.value;

    let min : u32;

    match args.min {
        Some(n) => min = n, 
        None => min = 0, // the default value is 0
    }

    if args.verbose{
        for i in min..=value {
            match fibo(i) {
                Some(n)    => println!("fibo({i}) = {}", n),
                None    => {println!("Overflow!"); break}
            }
        }
    } else {
        if fibo(value) == None {
            println!("Overflow!")
        } else {
            println!("fibo({value}) = {}", fibo(value).unwrap()) 
        }
    }
    
        //println!("fibo({i}) = {}",fibo(i).unwrap())
}

fn fibo(n: u32) -> Option<u32> {
    // Affichage des valeurs correctes uniquement
    if n<=1 {
        return Some(n);
    } else {
        let mut prev: u32 = 0;
        let mut current: u32 = 1;
        let mut o: Option<u32> = Some(1);

        for _ in 2..=n {
            o = prev.checked_add(current);
            if o != None {
                let next: u32 = o.unwrap();
                prev = current;
                current = next;
            }
        }
        o        
    }
}
    
    // Iterative implementation: Arithmétique saturée
    /* if n<=1 {
        n
    } else {
        let mut prev: u32 = 0;
        let mut current: u32 = 1;

        for _ in 2..=n {
            let next: u32 = prev.saturating_add(current);
            prev = current;
            current = next;
        }
        current        
    } */

    // Recursive implementation
    /* if n <= 1 {
        n
    } else {
        fibo(n-1) + fibo(n-2)
    } */
