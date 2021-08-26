use crate::{DRAW_SLEEP_TIME, MIN_DB, TERMINAL_WIDTH, TIME_WINDOWS};
use console::{style, StyledObject, Term};

use std::io::Error;
use terminal_size::{Height, Width};

const NUMBERS: [(i32, &str); 5] = [
    (0, "0"),
    (-10, "-10"),
    (-20, "-20"),
    (-30, "-30"),
    (-40, "-40"),
];

fn color<T>(db: f32, content: T) -> StyledObject<T> {
    let base = style(content);
    match db {
        _ if (-20.0..=-10.0).contains(&db) => base.yellow(),
        _ if (-10.0..=-0.0).contains(&db) => base.red(),
        _ => base.green(),
    }
}

fn draw(term: &Term) -> Result<(), Error> {
    let windows = TIME_WINDOWS.lock().unwrap();

    if windows.len() == 0 {
        panic!("Please set windows to draw");
    }

    let maxes = windows
        .iter()
        .map(|w| (w.keep_secs, w.max()))
        .collect::<Vec<_>>();

    drop(windows);

    term.clear_last_lines(maxes.len() + 3)?;
    println!("{}", style("Metering windows").underlined());

    for (seconds, max) in &maxes {
        println!("{:>4}s: {:.2} db", seconds, color(*max, max));
    }

    let percent = 1.0 - (maxes[0].1 / MIN_DB);
    let percent_max = 1.0 - (maxes.last().unwrap().1 / MIN_DB);

    let (Width(width), _) = terminal_size::terminal_size().unwrap_or((Width(80), Height(100)));

    let max_width = TERMINAL_WIDTH * width as f32;
    let num_blocks = percent * max_width;
    let max_block = percent_max * max_width;
    let skip = -MIN_DB / max_width;

    let mut current = MIN_DB;

    for _ in 0..num_blocks as usize {
        print!("{}", color(current, "█"));
        current += skip;
    }

    if num_blocks < max_block {
        for _ in 0..max_block as usize - num_blocks as usize {
            print!(" ");
            current += skip;
        }

        print!("{}", color(current, "█"));
    }

    println!();

    let mut number_line = vec![' '; max_width as usize + 1];
    for (position, string) in NUMBERS.iter() {
        let index = (max_width / MIN_DB) * (MIN_DB - *position as f32);
        let index = index as usize;

        if *position != 0
            && (index + 3 > max_width as usize
                || number_line[index - 2..index + 3].iter().any(|c| *c != ' '))
        {
            continue;
        }

        for i in 0..string.len() {
            let modifier = i as i32 - string.len() as i32 / 2;
            number_line[(index as i32 + modifier) as usize] = string.chars().nth(i).unwrap();
        }
    }

    for c in number_line {
        print!("{}", c);
    }
    println!();

    Ok(())
}

pub fn run() {
    let term = Term::stdout();
    loop {
        draw(&term).expect("Drawing failed");
        std::thread::sleep(DRAW_SLEEP_TIME);
    }
}
