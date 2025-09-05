use juste::genus::{Ctx, Highlight, ParseTable};

fn main() {
    let mut hg = Highlight {
        token_slice: [0, 0],
        parse_table: ParseTable { delimiter, table },
    };
    let mut c = Vec::<Ctx>::new();
    hg.test_highlight(
        &vec![
            b's', b't', b'r', b'u', b'c', b't', b' ', b'm', b'm', b' ', b' ',
        ],
        &mut c,
    );
    dbg!(c);
}

fn table(token: &[u8], idx: [usize; 2]) -> Ctx {
    match token {
        b"fn" => Ctx::Put { idx, col: 2 },
        b"struct" => Ctx::Future {
            idx,
            col_self: 2,
            col_next: 3,
        },
        b" " => Ctx::Gap,
        _ => Ctx::Hold { idx },
    }
}

fn delimiter(u: &u8) -> bool {
    match u {
        b' ' => true,
        _ => false,
    }
}
