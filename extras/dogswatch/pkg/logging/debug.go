package logging

// DebugEnable is a string passed in by the compiler to control the build's
// inclusion of Debuggable sections.
var DebugEnable string

// Debuggable means that the build should include any debugging logic in it. The
// compiler *should* erase anything that's otherwise in a conditional.
var Debuggable = DebugEnable != ""
