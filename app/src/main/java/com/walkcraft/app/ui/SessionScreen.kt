package com.walkcraft.app.ui

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState

@Composable
fun SessionScreen() {
    // [WC-10.3-UI-COLLECT-BEGIN]
    val ui = com.walkcraft.app.session.SessionBus.state.collectAsState(initial = com.walkcraft.app.session.UiSessionState())
    // Example binding:
    Text("Session steps: ${ui.value.steps}")
    // If you show elapsed time separately, prefer ui.value.elapsedMs (or your existing ticker)
    // [WC-10.3-UI-COLLECT-END]
}
