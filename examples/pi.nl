idx is 10000 in

0 loop {
  @idx! dup 
  2 % 0.5 swap -
  swap 0.5 +
  / +
  (@idx! - 1) dup -> @idx
  0 >= @loop jump?
}
@loop!

4 *
