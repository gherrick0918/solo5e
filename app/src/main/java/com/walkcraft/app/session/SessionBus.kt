// [WC-10.3-BUS-BEGIN]
package com.walkcraft.app.session

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

data class UiSessionState(
    val active: Boolean = false,
    val startTimeMs: Long = 0L,
    val elapsedMs: Long = 0L,
    val steps: Long = 0L
)

object SessionBus {
    private val _state = MutableStateFlow(UiSessionState())
    val state: StateFlow<UiSessionState> = _state

    fun publish(s: UiSessionState) {
        _state.value = s
    }
}
// [WC-10.3-BUS-END]
