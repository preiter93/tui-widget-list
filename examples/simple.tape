# To run this script, install vhs and run `vhs ./examples/simple.tape`
Output "examples/simple.gif"
Set Margin 10
Set Padding 2
Set BorderRadius 10
Set Width 1200
Set Height 300
Set PlaybackSpeed 1.0

Hide
Type "cargo run --example simple"
Enter
Sleep 0.5s
Show

Sleep 1.5s
Down@0.2s 20
Sleep 1.5s
