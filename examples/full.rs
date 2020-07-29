use demonstrate::demonstrate;

fn is_4() -> u8 {
    4
}

demonstrate! {
    describe module {
        before {
            let value1 = 4;
        }

        describe nested {
            it works {
                assert!(value1 == is_4())
            }
        }
    }
}
