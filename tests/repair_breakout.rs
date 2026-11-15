use mip_codex::{heuristic::Repair, mps};
use std::{
    io::Cursor,
    time::{Duration, Instant},
};
#[test]
fn violated_row_weights_escape_a_zero_gain_coordinate_minimum() {
    let model=mps::parse(Cursor::new("NAME plateau\nROWS\n N c\n G demand\n E same\nCOLUMNS\n x c 0 demand 1\n x same 1\n y c 0 demand 1\n y same -1\nRHS\n rhs demand 1 same 0\nBOUNDS\n BV b x\n BV b y\nENDATA\n")).unwrap();
    let mut repair = Repair::new(&model);
    let lower = [Some(0.); 2];
    let upper = [Some(1.); 2];
    assert!(
        repair
            .search(
                &[0.; 2],
                &lower,
                &upper,
                Instant::now() + Duration::from_secs(1),
                10000,
                1
            )
            .is_none()
    );
    repair.breakout = true;
    for _ in 0..2 {
        let x = repair
            .search(
                &[0.; 2],
                &lower,
                &upper,
                Instant::now() + Duration::from_secs(1),
                10000,
                1,
            )
            .unwrap();
        assert_eq!(x, vec![1., 1.]);
        assert!(repair.candidate_check(&x));
    }
    assert_eq!(repair.stats.penalty_updates, 2);
    repair.breakout = false;
    assert!(
        repair
            .search(
                &[0.; 2],
                &lower,
                &upper,
                Instant::now() + Duration::from_secs(1),
                10000,
                1
            )
            .is_none()
    );
}
#[test]
fn breakout_respects_work_and_cannot_invent_a_feasible_point() {
    let model=mps::parse(Cursor::new("NAME impossible\nROWS\n N c\n E zero\n E one\nCOLUMNS\n x c 0 zero 1\n x one 1\nRHS\n rhs zero 0 one 1\nBOUNDS\n BV b x\nENDATA\n")).unwrap();
    let mut repair = Repair::new(&model);
    repair.breakout = true;
    assert!(
        repair
            .search(
                &[0.],
                &[Some(0.)],
                &[Some(1.)],
                Instant::now() + Duration::from_secs(1),
                10000,
                1
            )
            .is_none()
    );
    assert!(repair.stats.penalty_updates > 0 && repair.stats.penalty_updates <= 32);
    assert!(
        repair
            .search(
                &[0.],
                &[Some(0.)],
                &[Some(1.)],
                Instant::now() + Duration::from_secs(1),
                0,
                1
            )
            .is_none()
    );
}
