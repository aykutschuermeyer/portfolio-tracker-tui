#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use crate::app::calc::fifo;

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
    fn it_works() {
        let (amounts, quantities) = set_sample_data();
        let result = fifo(amounts, quantities).unwrap();

        println!("Cumulative Units: {}", result.cumulative_units());
        println!("Cumulative Cost: {}", result.cumulative_cost());
        println!("Cost of Units Sold: {}", result.cost_of_units_sold());

        assert_eq!(result.cumulative_units().normalize(), dec!(80.0));
        assert_eq!(result.cumulative_cost().normalize(), dec!(7229.43));
        assert_eq!(result.cost_of_units_sold().normalize(), dec!(1777.02));
    }
}
