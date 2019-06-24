#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub enum Remainder {
    E0, E1, E2, E3, E4, E5, E6, E7,
}

impl Remainder {
    pub fn as_8ths(&self) -> usize {
        match self {
            &Remainder::E0 => 0,
            &Remainder::E1 => 1,
            &Remainder::E2 => 2,
            &Remainder::E3 => 3,
            &Remainder::E4 => 4,
            &Remainder::E5 => 5,
            &Remainder::E6 => 6,
            &Remainder::E7 => 7,
        }
    }

    pub fn from_8ths(n_8ths: usize) -> Self {
        match n_8ths % 8 {
            0 => Remainder::E0,
            1 => Remainder::E1,
            2 => Remainder::E2,
            3 => Remainder::E3,
            4 => Remainder::E4,
            5 => Remainder::E5,
            6 => Remainder::E6,
            7 => Remainder::E7,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Remainder;

    #[test]
    fn test_as_8ths() {
        assert_eq!(0, Remainder::E0.as_8ths());
        assert_eq!(1, Remainder::E1.as_8ths());
        assert_eq!(2, Remainder::E2.as_8ths());
        assert_eq!(3, Remainder::E3.as_8ths());
        assert_eq!(4, Remainder::E4.as_8ths());
        assert_eq!(5, Remainder::E5.as_8ths());
        assert_eq!(6, Remainder::E6.as_8ths());
        assert_eq!(7, Remainder::E7.as_8ths());
    }

    #[test]
    fn test_from_8ths() {
        const CYCLES: usize = 6;
        const ORDER: [Remainder; 8] = [
            Remainder::E0,
            Remainder::E1,
            Remainder::E2,
            Remainder::E3,
            Remainder::E4,
            Remainder::E5,
            Remainder::E6,
            Remainder::E7,
        ];

        let mut expected_cycle = ORDER.into_iter().cycle();

        for i in 0..=(CYCLES * 8) {
            let expected = *expected_cycle.next().unwrap();
            let produced = Remainder::from_8ths(i);

            assert_eq!(expected, produced);
        }
    }
}
