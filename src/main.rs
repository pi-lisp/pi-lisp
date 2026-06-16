mod cubical;

use crate::cubical::syntax::Term;
use crate::cubical::typechecker::{infer_closed, check_closed};
use crate::cubical::eval::eval;
use crate::cubical::interval::I;

fn main() {
    // 1. 인터벌 표현식 만들기: 0 OR (NOT 0)
    // Join( 0, Neg(0) ) -> 수학적으로 항상 1(참)이 되어야 합니다.
    let target_expr = I::Join(
        Box::new(I::I0),
        Box::new(I::Neg(Box::new(I::I0)))
    ); //

    // 2. Term으로 감싸기
    let interval_term = Term::TInterval(target_expr); //

    println!("--- 큐비컬 인터벌 TUP 테스트 ---");
    
    // 3. 계산 수행
    let result = eval(&interval_term); //

    println!("계산된 인터벌 결과: {:?}", result);
}