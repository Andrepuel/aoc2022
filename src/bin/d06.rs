use aoc::{input::Input, Answer};
use bitvec::prelude::BitArray;

const DAY: u32 = 6;

struct Protocol;
impl Protocol {
    fn start_of_packet_offset(input: &str) -> Option<usize> {
        Self::find_marker(input, 4)
    }

    fn start_of_message_offset(input: &str) -> Option<usize> {
        Self::find_marker(input, 14)
    }

    fn find_marker(input: &str, len: usize) -> Option<usize> {
        let input = input.as_bytes();
        if input.len() < len {
            return None;
        }

        (len..=input.len()).find(|&start| {
            let mut found = BitArray::<[u8; 256 / 8]>::ZERO;

            ((start - len)..start).all(|i| {
                let mut found_flag = found.get_mut(input[i] as usize).unwrap();
                let repeated = *found_flag;
                *found_flag = true;
                !repeated
            })
        })
    }
}

fn answer<I: Input>(input: I) -> aoc::Result<Answer<Vec<usize>>> {
    let size_hint = input.size_hint().1.unwrap_or_default();
    let mut packets = Vec::with_capacity(size_hint);
    let mut messages = Vec::with_capacity(size_hint);

    for input in input {
        let input = input?;
        packets.push(Protocol::start_of_packet_offset(&input).unwrap_or_default());
        messages.push(Protocol::start_of_message_offset(&input).unwrap_or_default());
    }

    Ok(Answer {
        part1: packets,
        part2: messages,
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
fn d06_example() {
    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer {
            part1: vec![7, 5, 6, 10, 11],
            part2: vec![19, 23, 23, 29, 26]
        }
    )
}
