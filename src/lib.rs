#[macro_use]
extern crate rust_decimal_macros;

use rust_decimal_macros::*;

use rust_decimal::Decimal;
use std::collections::HashMap;

use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Price {
    min: usize,
    price: Decimal,
}

impl Ord for Price {
    fn cmp(&self, other: &Self) -> Ordering {
        other.price.cmp(&self.price)
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Terminal {
    prices: HashMap<char, Vec<Price>>,
    items: HashMap<char, usize>,
}

impl Terminal {
    fn new(prices: HashMap<char, Vec<Price>>) -> Self {
        Terminal {
            prices: prices.iter().fold(HashMap::new(), |mut acc, (k, v)| {
                let mut nv = v.to_vec();

                nv.sort();

                acc.entry(*k).or_insert(nv);

                acc
            }),
            items: HashMap::new(),
        }
    }

    fn scan(&mut self, item: char) -> Result<(), ()> {
        if self.prices.get(&item).is_none() {
            panic!("invalid item {}", item);
        }

        let e = self.items.entry(item).or_insert(0);

        *e += 1;

        Ok(())
    }

    fn total(&self) -> Decimal {
        self.items.iter().fold(dec!(0), |mut acc, (item, count)| {
            acc += match self.prices.get(item) {
                Some(prices) => {
                    let mut item_total = dec!(0);

                    let mut c = *count;

                    for p in prices {
                        if c == 0 {
                            break;
                        }

                        if p.min == 0 {
                            item_total += p.price * Decimal::new(c as i64, 0);
                        } else if c >= p.min {
                            let x = c / p.min;

                            item_total += p.price * Decimal::new(x as i64, 0);

                            c -= x * p.min;
                        }
                    }

                    item_total
                }
                None => panic!(format!("bad item name {}", item)),
            };

            acc
        })
    }
}

#[macro_export]
macro_rules! setup_pricing(
    { $($key:literal => $($value:tt), + ); + } => {
        {
            let mut m = ::std::collections::HashMap::new();

            $(
                let mut v = vec![];

                $(
                    for iv in parse_price!($value) {
                        v.push(iv)
                    }
                )*;


                //TPDP" die if i more than one
                m.insert($key, v);
            )+

            Terminal::new(m)
        }
     };
);

#[macro_export]
macro_rules! parse_price(
    ($price:literal) => {
        {
            Price{ min: 0, price: dec!($price) };
        }
     };
    ([{ price: $price:literal }$(,)? $({ min: $bulk_quantity:literal, price: $bulk_price:literal }), *]) => {
        {
            let mut v = vec![];

            v.push(Price{ min: 0, price: dec!($price) });

            $(
              v.push(Price{ min: $bulk_quantity, price: dec!($bulk_price) });
             )*

                v
        }
     };
);

#[cfg(test)]
mod tests {
    use super::{Price, Terminal};

    #[test]
    fn it_parses() {
        assert_eq!(
            parse_price!([{ price: 2 }, { min: 4, price: 7 }]),
            vec![
                Price {
                    min: 0,
                    price: dec!(2)
                },
                Price {
                    min: 4,
                    price: dec!(7)
                }
            ]
        );
    }

    // Scan these items in this order: ABCDABAA; Verify the total price is $32.40.
    #[test]
    fn it_scans_even_splits() {
        let mut terminal = setup_pricing!('A' => [{ price: 2 }, { min: 4, price: 7 }]; 'B' => [{ price: 12 }]; 'C' => [{ price: 1.25 }, { min: 6, price: 6 }]; 'D' => [{ price: 0.15 }]);

        terminal.scan('A');
        terminal.scan('B');
        terminal.scan('C');
        terminal.scan('D');
        terminal.scan('A');
        terminal.scan('B');
        terminal.scan('A');
        terminal.scan('A');

        assert_eq!(terminal.total(), dec!(32.40));
    }

    // Scan these items in this order: CCCCCCC; Verify the total price is $7.25.
    #[test]
    fn it_scans_both_tiers() {
        let mut terminal = setup_pricing!('A' => [{ price: 2 }, { min: 4, price: 7 }]; 'B' => [{ price: 12 }]; 'C' => [{ price: 1.25 }, { min: 6, price: 6 }]; 'D' => [{ price: 0.15 }]);
        
        terminal.scan('C');
        terminal.scan('C');
        terminal.scan('C');
        terminal.scan('C');
        terminal.scan('C');
        terminal.scan('C');
        terminal.scan('C');

        assert_eq!(terminal.total(), dec!(7.25));
    }

    // Scan these items in this order: ABCD; Verify the total price is $15.40.
    #[test]
    fn it_scans_all_base() {
        let mut terminal = setup_pricing!('A' => [{ price: 2 }, { min: 4, price: 7 }]; 'B' => [{ price: 12 }]; 'C' => [{ price: 1.25 }, { min: 6, price: 6 }]; 'D' => [{ price: 0.15 }]);

        terminal.scan('A');
        terminal.scan('B');
        terminal.scan('C');
        terminal.scan('D');

        assert_eq!(terminal.total(), dec!(15.40));
    }
}

