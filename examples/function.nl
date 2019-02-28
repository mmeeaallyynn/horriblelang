"std.nl" include

Greeter is
	greet is name { _ }
		@name put

		"hello " print @name! @std::println!
		@name!
	in

	talk is name { _ }
		@name put

		"blabla, blabla" @std::println!
		@name!
	in

	say_goodbye is name { _ }
		@name put

		"bye " print @name! @std::println!
		 @name!
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

@Greeter::greet::name @std::println!