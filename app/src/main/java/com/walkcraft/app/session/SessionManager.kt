package com.walkcraft.app.session

import android.content.Context
import com.walkcraft.app.health.HcWriter
import com.walkcraft.app.sensors.StepReconciler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class SessionManager(
    private val context: Context,
    private val scope: CoroutineScope,
    private val settings: Settings
) {
    data class Settings(
        val writeToHealthConnect: Boolean,
        val writeStepsIfMissing: Boolean
    )

    // [WC-10.3-RECONCILE-WIRE-BEGIN]
    // Create once (field):
    private val reconciler by lazy { StepReconciler(context) }
    // [WC-10.3-RECONCILE-WIRE-END]

    private val _state = MutableStateFlow(UiSessionState())
    val state: StateFlow<UiSessionState> = _state

    private var tickerJob: Job? = null
    private var sessionNote: String? = null
    private val minuteBuckets = mutableMapOf<Long, Int>()

    fun startSession(note: String? = null) {
        tickerJob?.cancel()
        minuteBuckets.clear()
        sessionNote = note
        // [WC-10.3-START-RESET-BEGIN]
        val now = System.currentTimeMillis()
        // Replace with your actual state holder fields
        _state.value = _state.value.copy(
            active = true,
            startTimeMs = now,
            elapsedMs = 0L,
            steps = 0L
        )
        com.walkcraft.app.session.SessionBus.publish(
            com.walkcraft.app.session.UiSessionState(
                active = true,
                startTimeMs = now,
                elapsedMs = 0L,
                steps = 0L
            )
        )
        // Ensure your StepTracker baseline is reset here too (if applicable)
        // [WC-10.3-START-RESET-END]

        reconciler.start()

        tickerJob = scope.launch {
            while (isActive && _state.value.active) {
                delay(1000L)
                val elapsed = System.currentTimeMillis() - _state.value.startTimeMs
                _state.value = _state.value.copy(elapsedMs = elapsed)
                // [WC-10.3-PUBLISH-BEGIN]
                com.walkcraft.app.session.SessionBus.publish(
                    com.walkcraft.app.session.UiSessionState(
                        active = _state.value.active,
                        startTimeMs = _state.value.startTimeMs,
                        elapsedMs = _state.value.elapsedMs,
                        steps = _state.value.steps
                    )
                )
                // [WC-10.3-PUBLISH-END]

                val seconds = (elapsed / 1000L).toInt()
                if (seconds > 0 && seconds % 15 == 0) {
                    val correction = reconciler.boundedCorrection(_state.value.steps)
                    if (correction > 0) {
                        _state.value = _state.value.copy(steps = _state.value.steps + correction)
                        // [WC-10.3-PUBLISH-BEGIN]
                        com.walkcraft.app.session.SessionBus.publish(
                            com.walkcraft.app.session.UiSessionState(
                                active = _state.value.active,
                                startTimeMs = _state.value.startTimeMs,
                                elapsedMs = _state.value.elapsedMs,
                                steps = _state.value.steps
                            )
                        )
                        // [WC-10.3-PUBLISH-END]
                    }
                }
            }
        }
    }

    fun onDetectorStep(timestampMillis: Long = System.currentTimeMillis(), steps: Int = 1) {
        if (!_state.value.active || steps <= 0) return
        _state.value = _state.value.copy(steps = _state.value.steps + steps)
        // [WC-10.3-PUBLISH-BEGIN]
        com.walkcraft.app.session.SessionBus.publish(
            com.walkcraft.app.session.UiSessionState(
                active = _state.value.active,
                startTimeMs = _state.value.startTimeMs,
                elapsedMs = _state.value.elapsedMs,
                steps = _state.value.steps
            )
        )
        // [WC-10.3-PUBLISH-END]

        val minute = timestampMillis / 60_000L
        minuteBuckets[minute] = (minuteBuckets[minute] ?: 0) + steps
    }

    suspend fun stopSession() {
        if (!_state.value.active) return
        tickerJob?.cancel()
        tickerJob = null
        val endMs = System.currentTimeMillis()

        // [WC-10.3-RECONCILE-WIRE-BEGIN]
        reconciler.stop()
        // [WC-10.3-RECONCILE-WIRE-END]

        val finalState = _state.value.copy(
            active = false,
            elapsedMs = endMs - _state.value.startTimeMs
        )
        _state.value = finalState
        // [WC-10.3-PUBLISH-BEGIN]
        com.walkcraft.app.session.SessionBus.publish(
            com.walkcraft.app.session.UiSessionState(
                active = _state.value.active,
                startTimeMs = _state.value.startTimeMs,
                elapsedMs = _state.value.elapsedMs,
                steps = _state.value.steps
            )
        )
        // [WC-10.3-PUBLISH-END]

        val minutesPayload = minuteBuckets.entries
            .sortedBy { it.key }
            .map { (minute, steps) ->
                HcWriter.MinuteSteps(epochMinute = minute, steps = steps)
            }
        minuteBuckets.clear()

        if (settings.writeToHealthConnect) {
            val snapshot = HcWriter.LocalSessionSnapshot(
                startMs = finalState.startTimeMs,
                endMs = endMs,
                steps = finalState.steps,
                note = sessionNote
            )
            withContext(Dispatchers.IO) {
                // [WC-10.3-WRITE-CALL-BEGIN]
                val snap = com.walkcraft.app.health.HcWriter.LocalSessionSnapshot(
                    startMs = snapshot.startMs,              // adapt to your names
                    endMs   = snapshot.endMs,
                    steps   = snapshot.steps,
                    note    = snapshot.note
                )
                val minutes = minutesPayload.map { m ->
                    com.walkcraft.app.health.HcWriter.MinuteSteps(
                        epochMinute = m.epochMinute,
                        steps = m.steps
                    )
                }
                com.walkcraft.app.health.HcWriter.write(
                    ctx = context, // if in a Service; otherwise pass a Context
                    local = snap,
                    minutes = minutes,
                    writeStepsIfMissing = settings.writeStepsIfMissing
                )
                // [WC-10.3-WRITE-CALL-END]
            }
        }
    }
}
