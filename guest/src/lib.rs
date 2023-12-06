wit_bindgen::generate!( {
    world: "main",
    exports: {
        world: Host
    }
});

struct Host;

impl Guest for Host {
    fn run(args: Vec<String>) -> Result<i32, String> {
        if args == ["guest", "Hello"] {
            // print("Hello from the other side");
        } else {
            return Err("Invalid arguments".into());
        }

        let mut items = Vec::new();
        for i in 0..10 {
            items.push(i);
        }

        let (sq, sqrt) = get_value(16);
        assert_eq!(sq, 256);
        assert_eq!(sqrt as u32, 4);

        print(&format!("Hello from guest {items:?}"));
        Ok(42)
    }

    fn get_name() -> String {
        "guest-module".into()
    }
}
