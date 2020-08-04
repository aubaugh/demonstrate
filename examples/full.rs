use demonstrate::demonstrate;

fn is_4() -> u8 {
    4
}

demonstrate! {
    describe module {
        use super::is_4;

        before {
            let four = 4;
        }

        #[should_panic]
        it can_fail {
            assert!(four != 4)
        }

        test is_returnable -> Result<(), &'static str> {
            if is_4() == four {
                Ok(())
            } else {
                Err("It isn't 4! :o")
            }
        }

        #[async_attributes::test]
        async context asynchronous {
            before {
                let is_4_task = async_std::task::spawn(async {
                    is_4()
                });
            }

            it awaits {
                assert_eq!(four, is_4_task.await)
            }
        }
    }
}
