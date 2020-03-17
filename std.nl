std {
  if is
    @then jump?
  in
  endl is
    "\n" print
  in
  println is
    print @std::endl!
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

  list {
    guard is "|" in

    fold is 
      func is _ in
      accu is _ in

      -> @func

      loop_start {
        dup @std::list::guard! != then {
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

      @std::list::guard!
      loop_start {
        (from < to) then {
          @from!
          (from + step) -> @from
          @loop_start!
        } @std::if!
      } @loop_start!
    in
  }
}

