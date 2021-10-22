// place a guard at the beginning of the stack to detect possible underflow errors
STACK_START

// toplevel drop that doesn't remove the stack guard
"drop" is
  dup STACK_START != lambda
    drop
  in jump?
in


generator is
  // consumes initial initial value, function point, reference to memory with 3 spaces 
  // produces reference to memory
  // write params for new generator a specified memory location
  new is
    -> @generator::value
    -> @generator::func
    -> @generator::ref 
    "gen" @generator::ref! 2 + put
    @generator::value! @generator::ref! 1 + put
    @generator::func! @generator::ref! put
    @generator::ref!
  in
  // load a generator from an address
  load is
    -> @generator::ref 
    @generator::ref!!
    "gen" != dup lambda "not a generator!" print in jump?
    not lambda
      -> @generator::value
      -> @generator::func
    in jump?
  in
  // save the state of the current generator
  save is
    @generator::ref!
    @generator::func!
    @generator::value!
    @generator::new!
    drop
  in
  // consumes integer number
  // produce the next n items as a list
  take is
    | swap 
    loop {
      1 - dup
      0 >= lambda
        @generator::next!
        swap
        @loop!
      in jump?
    } @loop! 

    drop
  in

  func is _ in
  value is 0 in
  ref is _ in
  // called to generate the next value
  next is
    @generator::value! dup @generator::func!!
    -> @generator::value
  in
in


memory {
  mem is _100 in
  idx is 0 in

  alloc is
    @memory::mem @memory::idx! +
    swap
    @memory::idx! + -> @memory::idx
  in
}

std {
  op {
    "+" is + in
    "-" is - in
    "*" is * in
    "/" is / in
    "drop" is drop in
  }
  if is
    @then jump?
  in
  ifelse is
    iftrue is _ in
    iffalse is _ in

    -> @iffalse
    -> @iftrue

    @iffalse$ swap lambda drop @iftrue! in jump? jump
  in

  endl is
    "\n" print
  in
  println is
    print
  in
  dec is
    ref is 0 in @ref put
    @ref! jump 1 - @ref! put
  in
  inc is
    ref is 0 in @ref put
    @ref! jump 1 + @ref! put
  in
  alert is
    "alert(\"" swap "\")" + + __jseval drop
  in
  prompt is 
    "prompt(\"" swap "\")" + + __jseval
  in
  dropall is
    STACK_START -> @lists::foreach::guard
    lambda drop in @lists::foreach!
    "|" -> @lists::foreach::guard
    STACK_START
  in

  pow is
    base is _ in
    exp  is _ in
    val  is 1 in
    op   is _ in

    1 -> @val
    @std::op::* -> @op

    -> @exp
    -> @base

    (@exp! < 0) lambda
      @std::op::/ -> @op
      (@exp! * -1) -> @exp
    in jump?

    0 @exp! @std::range! 
    lambda
      drop
      @val! @base! @op! jump -> @val
    in @std::foreach!

    @val!
  in
}

lists {
  len is
    idx is -1 in
    -1 -> @idx
    loop {
      @idx! pull "|" != lambda
        @idx @std::dec!
        @loop!
      in jump?
    } @loop!
    @idx! 1 + -1 *
  in
  fold is 
    guard is "|" in
    func is _ in
    accu is _ in

    -> @accu
    -> @func

    loop_start {
      dup @guard! != then {
        @accu! @func! jump -> @accu
        @loop_start!
      } @std::if!
    } @loop_start!

    drop
    @accu!
  in
  range is 
    from is _ in
    to   is _ in
    step is 1 in
    -> @to
    -> @from

    |
    loop_start {
      (@from! < @to!) then {
        (@to! - @step!) -> @to
        @to!
        @loop_start!
      } @std::if!
    } @loop_start!
  in
  foreach is
    guard is "|" in
    func is _ in

    -> @func

    loop_start {
      dup @guard! != then {
        @func! jump
        @loop_start!
      } @std::if!
    } @loop_start!

    drop
  in
  join is 
    guard is "|" in
    sep is _ in
    accu is "" in

    -> @sep
    -> @accu

    loop_start {
      dup @guard! != then {
        @sep! @accu! + + -> @accu
        @loop_start!
      } @std::if!
    } @loop_start!

    drop
    @accu!
  in
  "drop" is
    lambda drop in @lists::foreach!
  in
  string is 
    \space @lists::join!
  in

  find is
    best is _ in
    space is _ in
    op is _ in
    -> @lists::find::op
    -> @lists::find::best

    lambda 
      dup -> @lists::find::space

      @lists::find::best$ @op$! lambda 
        @lists::find::space$ -> @lists::find::best
      in jump?
    in @lists::foreach!

    @lists::find::best$
  in

  max is
    lambda > in @lists::find!
  in

  min is
    lambda < in @lists::find!
  in

  "dup" is 
    len is _ in
    idx is -1 in
    -1 -> @idx
    @lists::len! -1 * 1 - -> @lists::dup::len
    | 
    loop {
      @lists::dup::len$ pull
      @lists::dup::idx$ 1 - @lists::dup::len$ > lambda
        @lists::dup::idx @std::dec!
        @loop!
      in jump?
    } @loop!
  in

  minmax is
    max is _ in
    @lists::dup! @lists::max! -> @max @lists::min! @lists::minmax::max$
  in
}

