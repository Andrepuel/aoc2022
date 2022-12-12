use std::{
    collections::{HashMap, VecDeque},
    iter::Peekable,
    num::ParseIntError,
    str::FromStr,
};

use aoc::{
    input::{Input, InputError},
    Answer,
};

const DAY: u32 = 11;

#[derive(Debug, Clone)]
struct Monkey<T: Item = SimpleItem> {
    items: VecDeque<T>,
    operation: Operation,
    test: Test,
}
impl Monkey<SimpleItem> {
    fn from_tree(tree: &Tree) -> MonkeyResult<Self> {
        const STARTING: &str = "Starting items";
        const OPERATION: &str = "Operation";
        const TEST: &str = "Test";

        let items = tree.get(STARTING)?.get_value()?;
        let items = items
            .split(',')
            .map(|x| x.trim())
            .map(|x| Ok(SimpleItem(x.parse()?)))
            .collect::<Result<_, _>>()
            .map_err(|e| MonkeyError::InvalidItem(items.to_string(), e))?;

        let operation = tree.get(OPERATION)?.get_value()?.parse()?;

        let test = tree.get(TEST)?.get_subtree()?;
        let test = Test::from_tree(test.0, test.1)?;

        Ok(Monkey {
            items,
            operation,
            test,
        })
    }
}

impl<T: Item> Monkey<T> {
    fn inspect(&mut self) -> Option<(usize, T)> {
        let mut item = self.items.pop_front()?;
        item.operate(self.operation);
        item.relax();

        let to = self.test.test(&item);

        Some((to, item))
    }

    fn receive(&mut self, item: T) {
        self.items.push_back(item);
    }

    fn map<U: Item, F: FnMut(&T) -> U>(&self, f: F) -> Monkey<U> {
        let items = self.items.iter().map(f).collect();

        Monkey {
            items,
            operation: self.operation,
            test: self.test,
        }
    }
}

trait Item {
    fn relax(&mut self);
    fn divisible(&self, by: u32) -> bool;
    fn operate(&mut self, operation: Operation);
}

#[derive(Debug, Clone)]
struct SimpleItem(u32);
impl Item for SimpleItem {
    fn relax(&mut self) {
        self.0 /= 3;
    }

    fn divisible(&self, by: u32) -> bool {
        self.0 % by == 0
    }

    fn operate(&mut self, operation: Operation) {
        self.0 = match operation {
            Operation::Square => self.0 * self.0,
            Operation::Multiply(right) => self.0 * right,
            Operation::Sum(right) => self.0 + right,
        }
    }
}

#[derive(Clone)]
struct ModuleItem {
    values: Vec<(u32, u32)>,
}
impl std::fmt::Debug for ModuleItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ModuleItem")
            .field(&self.values /* .last().unwrap().0*/)
            .finish()
    }
}
impl ModuleItem {
    fn new<I: IntoIterator<Item = u32>>(base: u32, modulos: I) -> ModuleItem {
        let values = modulos
            .into_iter()
            .map(|modulo| (base % modulo, modulo))
            .collect();

        ModuleItem { values }
    }
}
impl Item for ModuleItem {
    fn relax(&mut self) {}

    fn divisible(&self, by: u32) -> bool {
        self.values
            .iter()
            .find(|(_, modulo)| *modulo == by)
            .unwrap()
            .0
            == 0
    }

    fn operate(&mut self, operation: Operation) {
        match operation {
            Operation::Square => {
                for (value, modulo) in self.values.iter_mut() {
                    *value = ((*value) * (*value)) % *modulo;
                }
            }
            Operation::Multiply(right) => {
                for (value, modulo) in self.values.iter_mut() {
                    *value = ((*value) * (right % *modulo)) % *modulo;
                }
            }
            Operation::Sum(right) => {
                for (value, modulo) in self.values.iter_mut() {
                    *value = ((*value) + (right % *modulo)) % *modulo;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Square,
    Multiply(u32),
    Sum(u32),
}
impl FromStr for Operation {
    type Err = OperationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix("new = ")
            .ok_or_else(|| OperationError::InvalidPrefix(s.to_string()))?;

        let mut comps = s.splitn(3, ' ');
        let left = comps.next().unwrap_or_default();
        let operator = comps.next().unwrap_or_default();
        let right = comps.next().unwrap_or_default();

        let left: Operand = left.parse()?;
        let right: Operand = right.parse()?;

        Ok(match (left, operator, right) {
            (Operand::Old, "+", Operand::Number(right)) => Operation::Sum(right),
            (Operand::Old, "*", Operand::Number(right)) => Operation::Multiply(right),
            (Operand::Old, "*", Operand::Old) => Operation::Square,
            _ => return Err(OperationError::InvalidOperation(s.to_string())),
        })
    }
}

#[derive(Debug, Clone)]
enum Operand {
    Number(u32),
    Old,
}
impl FromStr for Operand {
    type Err = OperationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "old" {
            return Ok(Operand::Old);
        }

        Ok(Operand::Number(s.parse()?))
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Test {
    divisible: u32,
    if_true: usize,
    if_false: usize,
}
impl Test {
    fn from_tree(main: &str, tree: &Tree) -> Result<Self, TestError> {
        const DIVISIBLE: &str = "divisible by ";
        const THROW: &str = "throw to monkey ";
        const IF_TRUE: &str = "If true";
        const IF_FALSE: &str = "If false";

        let divisible = main
            .strip_prefix(DIVISIBLE)
            .ok_or_else(|| TestError::InvalidPrefix(DIVISIBLE, main.to_string()))?;
        let divisible = divisible.parse()?;

        let if_true = tree.get(IF_TRUE)?.get_value()?;
        let if_true = if_true
            .strip_prefix(THROW)
            .ok_or_else(|| TestError::InvalidPrefix(THROW, if_true.to_string()))?
            .parse()?;

        let if_false = tree.get(IF_FALSE)?.get_value()?;
        let if_false = if_false
            .strip_prefix(THROW)
            .ok_or_else(|| TestError::InvalidPrefix(THROW, if_false.to_string()))?
            .parse()?;

        Ok(Test {
            divisible,
            if_true,
            if_false,
        })
    }

    fn test<T: Item>(&self, item: &T) -> usize {
        match item.divisible(self.divisible) {
            true => self.if_true,
            false => self.if_false,
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum MonkeyError {
    #[error("{0}")]
    MissingAttribute(#[from] MissingAttribute),
    #[error("{0}")]
    ExtraneousSubtree(#[from] ExtraneousSubtree),
    #[error("{0}")]
    MissingSubtree(#[from] MissingSubtree),
    #[error("{1}, invalid number on item list {0:?}")]
    InvalidItem(String, ParseIntError),
    #[error("{0}")]
    Operation(#[from] OperationError),
    #[error("{0}")]
    TestError(#[from] TestError),
}
impl From<MonkeyError> for aoc::Error {
    fn from(value: MonkeyError) -> Self {
        aoc::Error::Semantic(value.into())
    }
}
type MonkeyResult<T> = Result<T, MonkeyError>;

#[derive(thiserror::Error, Debug)]
enum OperationError {
    #[error("Operation must start with {pre:?}, got {0:?}", pre = "new = ")]
    InvalidPrefix(String),
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Invalid operation: {0:?}")]
    InvalidOperation(String),
}

#[derive(thiserror::Error, Debug)]
enum TestError {
    #[error("Invalid prefix {1:?}, expected {0:?}")]
    InvalidPrefix(&'static str, String),
    #[error("{0}")]
    MissingAttribute(#[from] MissingAttribute),
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("{0}")]
    ExtraneousSubtree(#[from] ExtraneousSubtree),
}

#[derive(Debug)]
struct Tree {
    attributes: HashMap<String, TreeValue>,
}
impl Tree {
    fn input<I: Input>(input: I) -> impl Iterator<Item = ParseResult<(String, Tree)>> {
        let mut input = input.peekable();

        std::iter::from_fn(move || {
            let name = loop {
                match input.next()? {
                    Ok(name) if name.is_empty() => continue,
                    Ok(name) => break name,
                    Err(e) => return Some(Err(e.into())),
                }
            };

            let name = match name.split_once(':') {
                Some((name, _)) => name,
                None => return Some(Err(ParseError::MissingSeparator(name.to_string()))),
            };

            Self::next_tree(&mut input, 1)
                .transpose()
                .map(|r| r.map(|tree| (name.to_string(), tree)))
        })
    }

    fn next_tree<I: Input>(
        input: &mut Peekable<I>,
        expected_level: usize,
    ) -> ParseResult<Option<Self>> {
        let mut attributes = HashMap::default();
        while let Some((key, value)) = Self::next_attribute(input, expected_level)? {
            attributes.insert(key, value);
        }

        Ok(match !attributes.is_empty() {
            true => Some(Tree { attributes }),
            false => None,
        })
    }

    fn next_attribute<I: Input>(
        input: &mut Peekable<I>,
        expected_level: usize,
    ) -> ParseResult<Option<(String, TreeValue)>> {
        let next = match input.peek() {
            Some(Ok(next)) => next,
            Some(Err(_)) => {
                return Err(input.next().unwrap().err().unwrap().into());
            }
            None => return Ok(None),
        };

        if next.is_empty() {
            return Ok(None);
        }

        let level = next
            .bytes()
            .into_iter()
            .enumerate()
            .find(|(_, value)| *value != b' ')
            .map(|(idx, _)| idx)
            .unwrap_or_default()
            / 2;
        if level < expected_level {
            return Ok(None);
        }
        if level > expected_level {
            return Err(ParseError::UnexpectedIndentation(next.to_string()));
        }

        let next = input.next().unwrap().unwrap();
        let (key, value) = next
            .split_once(':')
            .ok_or_else(|| ParseError::MissingSeparator(next.clone()))?;

        let key = key.trim().to_string();
        let value = value.trim().to_string();
        let subtree = Self::next_tree(input, expected_level + 1)?;
        let value = match subtree {
            Some(subtree) => TreeValue::Subtree(value, Box::new(subtree)),
            None => TreeValue::Value(value),
        };

        Ok(Some((key, value)))
    }

    fn get(&self, attribute: &'static str) -> Result<&TreeValue, MissingAttribute> {
        self.attributes
            .get(attribute)
            .ok_or(MissingAttribute(attribute))
    }
}

#[derive(Debug)]
enum TreeValue {
    Value(String),
    Subtree(String, Box<Tree>),
}
impl TreeValue {
    fn get_value(&self) -> Result<&str, ExtraneousSubtree> {
        match self {
            TreeValue::Value(value) => Ok(value),
            TreeValue::Subtree(_, _) => Err(ExtraneousSubtree),
        }
    }

    fn get_subtree(&self) -> Result<(&str, &Tree), MissingSubtree> {
        match self {
            TreeValue::Value(_) => Err(MissingSubtree),
            TreeValue::Subtree(main, tree) => Ok((main, tree.as_ref())),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Missing {0:?} attribute")]
struct MissingAttribute(&'static str);

#[derive(thiserror::Error, Debug)]
#[error("Unexpected subtree")]
struct ExtraneousSubtree;

#[derive(thiserror::Error, Debug)]
#[error("Missing subtree")]
struct MissingSubtree;

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Unexpected indentation at {0:?}")]
    UnexpectedIndentation(String),
    #[error("Missing {sep:?} separator at {0:?}", sep = ':')]
    MissingSeparator(String),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

fn answer<I: Input>(input: I) -> aoc::Result<Answer<usize>> {
    let monkeys = Tree::input(input)
        .map(|tree| aoc::Result::Ok(Monkey::from_tree(&tree?.1)?))
        .collect::<Result<Vec<_>, _>>()?;

    let stressed_monkey_business;
    {
        let modules = monkeys.iter().map(|x| x.test.divisible);

        let mut monkeys = monkeys
            .iter()
            .map(|monkey| monkey.map(|item| ModuleItem::new(item.0, modules.clone())))
            .collect::<Vec<_>>();

        let mut activity = vec![0; monkeys.len()];

        for _ in 0..10_000 {
            for idx in 0..monkeys.len() {
                while let Some((to, item)) = monkeys[idx].inspect() {
                    activity[idx] += 1;
                    monkeys[to].receive(item);
                }
            }
        }

        let mut best_activity = [0, 0];
        for mut each in activity {
            for best in best_activity.iter_mut() {
                if each > *best {
                    std::mem::swap(&mut each, best);
                }
            }
        }

        stressed_monkey_business = best_activity[0] * best_activity[1];
    }

    let monkey_business;
    {
        let mut monkeys = monkeys;
        let mut activity = vec![0; monkeys.len()];

        for _ in 0..20 {
            for idx in 0..monkeys.len() {
                while let Some((to, item)) = monkeys[idx].inspect() {
                    activity[idx] += 1;
                    monkeys[to].receive(item);
                }
            }
        }

        let mut best_activity = [0, 0];
        for mut each in activity {
            for best in best_activity.iter_mut() {
                if each > *best {
                    std::mem::swap(&mut each, best);
                }
            }
        }

        monkey_business = best_activity[0] * best_activity[1];
    }

    Ok(Answer {
        part1: monkey_business,
        part2: stressed_monkey_business,
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
fn d11_test() {
    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer {
            part1: 10605,
            part2: 2713310158
        }
    )
}
