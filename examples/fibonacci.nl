"std.nl" include

fibonacci is
	a is 0 in
	b is 1 in
	result is 0 in

	@a$ @b$ + -> @result
	@b$ -> @a
	@result$ -> @b
	1 - 
	dup 0 >
	loop?
	@result$
in

50 @fibonacci! print
