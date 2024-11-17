use std::env;
use folds::StampFolder;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("Usage: [res/mod] dimension...");
        return;
    }

    let (res, mod_val, args_used) = if args[0].contains('/') {
        let parts: Vec<&str> = args[0].split('/').collect();
        (
            parts[0].parse::<i32>().unwrap(),
            parts[1].parse::<i32>().unwrap(),
            1,
        )
    } else {
        (0, 0, 0)
    };

    let dimensions: Vec<i32> = args
        .iter()
        .skip(args_used)
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    let mut folder = StampFolder::new();
    folder.foldings(&dimensions, true, res, mod_val);
    println!("{}", folder.count);
}
