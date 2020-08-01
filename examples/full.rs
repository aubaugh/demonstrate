use demonstrate::demonstrate;

fn is_4() -> u8 {
    4
}

demonstrate! {
    it is_root {
        assert!(true)
    }

    describe module {
        before {
            let four = 4;
        }

        test is_returnable -> Result<(), String> {
            if is_4() == four {
                Ok(())
            } else {
                Err("It isn't 4 :o".to_string())
            }
        }

        #[async_attributes::test]
        async context asynchronous {
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
