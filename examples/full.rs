use demonstrate::demonstrate;

fn is_4() -> u8 {
    4
}

demonstrate! {
    describe module {
        before {
            let value1 = 3;
        }

        #[should_panic]
        describe fails {
            it once {
                assert!(value1 == is_4())
            }
            it twice {
                assert!(false)
            }
        }
    }
}
