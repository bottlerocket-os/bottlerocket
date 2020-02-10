// Agent communicates bidirectionally with the host platform - Bottlerocket -
// and its managing controller to execute update operations in a coordinated
// manner. The Agent is responsible for publishing its update state, host
// state, and executing on permitted actions as indicated by the Controller.
//
// The Agent is intentionally simplistic in that it makes no decision about its
// next steps short of interpreting what's communicated by way of agreed upon
// state transition indicators.
package agent
