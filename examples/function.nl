"std.nl" include

Greeter is
	greet is
		dup
		"hello " print
		@std::println!
	in

	talk is
		"blabla, blabla" @std::println!
	in

	say_goodbye is
		dup
		"bye " print
		@std::println!
	in
in

"Gunther"
	@Greeter::greet!
	@Greeter::talk!
	@Greeter::say_goodbye!

	@Greeter::greet!
	@Greeter::say_goodbye!
drop

"\n" print

@Greeter!
"Werner"
	@greet!
	@talk!
	@say_goodbye!
drop

