"std.nl" include

/*class*/ Person {
	new /* var, name */ {
		varname is "" in @varname put

		@varname! priv
			name is "" in
		in
	}
}

"gunther" @Person::new!
"Günther" @gunther::name put

"harald" @Person::new!
"Harald" @harald::name put

@gunther::name! @std::println!
