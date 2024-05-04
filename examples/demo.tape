# To run this script, install vhs and run `vhs ./examples/demo.tape`
Output "examples/demo.gif"
Set Margin 10
Set Padding 2
Set BorderRadius 10
Set Width 1200
Set Height 700
Set PlaybackSpeed 1.0

Hide
Type "cargo run --example demo"
Enter
Sleep 0.5s
Show

Sleep 1.5s
Down@0.5s 7
Sleep 0.5s
Right@0.5s 11
Sleep 1.5s
