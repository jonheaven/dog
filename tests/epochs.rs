use {super::*, dog::subcommand::epochs::Output, ordinals::Koinu};

#[test]
fn empty() {
  assert_eq!(
    CommandBuilder::new("epochs").run_and_deserialize_output::<Output>(),
    Output {
      starting_sats: vec![
        Koinu(0),
        Koinu(1050000000000000),
        Koinu(1575000000000000),
        Koinu(1837500000000000),
        Koinu(1968750000000000),
        Koinu(2034375000000000),
        Koinu(2067187500000000),
        Koinu(2083593750000000),
        Koinu(2091796875000000),
        Koinu(2095898437500000),
        Koinu(2097949218750000),
        Koinu(2098974609270000),
        Koinu(2099487304530000),
        Koinu(2099743652160000),
        Koinu(2099871825870000),
        Koinu(2099935912620000),
        Koinu(2099967955890000),
        Koinu(2099983977420000),
        Koinu(2099991988080000),
        Koinu(2099995993410000),
        Koinu(2099997995970000),
        Koinu(2099998997250000),
        Koinu(2099999497890000),
        Koinu(2099999748210000),
        Koinu(2099999873370000),
        Koinu(2099999935950000),
        Koinu(2099999967240000),
        Koinu(2099999982780000),
        Koinu(2099999990550000),
        Koinu(2099999994330000),
        Koinu(2099999996220000),
        Koinu(2099999997060000),
        Koinu(2099999997480000),
        Koinu(2099999997690000)
      ]
    }
  );
}
