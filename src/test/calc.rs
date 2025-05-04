#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use crate::app::calc::calculate_position_state;

    fn set_sample_data() -> (Vec<Decimal>, Vec<Decimal>) {
        let amounts: Vec<Decimal> = vec![
            dec!(-1777.02),
            dec!(-1659.08),
            dec!(-2190.06),
            dec!(-1768.21),
            dec!(-1612.08),
            dec!(2275.64),
        ];
        let quantities: Vec<Decimal> = vec![
            dec!(20.00),
            dec!(20.00),
            dec!(20.00),
            dec!(20.00),
            dec!(20.00),
            dec!(-20.00),
        ];

        (amounts, quantities)
    }

    #[test]
    fn fifo_works() {
        let (amounts, quantities) = set_sample_data();
        let result = calculate_position_state(amounts, quantities).unwrap();

        println!("Result: {:#?}", result);

        assert_eq!(result.cumulative_units().normalize(), dec!(80.0));
        assert_eq!(result.cumulative_cost().normalize(), dec!(7229.43));
        assert_eq!(result.cost_of_units_sold().normalize(), dec!(1777.02));
    }
}
