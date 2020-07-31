use demonstrate::demonstrate;

fn is_4() -> u8 {
    4
}

demonstrate! {
    describe module {
        before {
            let four = 4;
        }

        it is_four {
            assert_eq!(is_4(), four)
        }

        #[async_attributes::test]
        async describe asynchronous {
            before {
                let add_task = async_std::task::spawn(async {
                    1 + 1 + 1 + 1
                });
            }

            it awaits {
                assert_eq!(four, add_task.await)
            }
        }

        #[should_panic]
        it can_fail {
            assert!(four != 4)
        }
    }
}
