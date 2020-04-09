# STM32F0 time counter

Shows the elapsed time in HH:MM:SS format, until it overflows (after 18 hours)

Time keeping is done with an interrupt. Every time the interrupt fires, the counter's current value
is displayed and then updated. Uses SSD1306 OLED in TerminalMode.

Pressing the button resets the timer.

STM32F0xx requires the peripherals to be defined within the Critical Section. If the display update was done in a loop within the CS,
this wouldn't work, as the interrupt wouldn't fire. Moving the loop outside wouldn't work, either, as the display instance wouldn't be
in the scope. This solves the problem.

Dev board: STMF051C8T6

