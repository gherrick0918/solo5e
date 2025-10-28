// [WC-10.3-RECONCILER-BEGIN]
package com.walkcraft.app.sensors

import android.content.Context
import android.hardware.Sensor
import android.hardware.SensorEvent
import android.hardware.SensorEventListener
import android.hardware.SensorManager
import kotlin.math.max
import kotlin.math.min

class StepReconciler(context: Context) : SensorEventListener {
    private val sm = context.getSystemService(Context.SENSOR_SERVICE) as SensorManager
    private val counter = sm.getDefaultSensor(Sensor.TYPE_STEP_COUNTER)

    private var baseline: Float? = null
    @Volatile private var lastCounter: Float? = null

    fun start() {
        baseline = null
        lastCounter = null
        counter?.let { sm.registerListener(this, it, SensorManager.SENSOR_DELAY_NORMAL) }
    }

    fun stop() {
        sm.unregisterListener(this)
        baseline = null
        lastCounter = null
    }

    /**
     * Given sessionSteps computed from TYPE_STEP_DETECTOR, returns a small correction
     * so UI/notification stay accurate even if detector misses a few.
     * We bound corrections to [0..20] to avoid spikes; call at ~15s cadence.
     */
    fun boundedCorrection(sessionSteps: Long): Long {
        val c = lastCounter ?: return 0L
        val b = baseline ?: return 0L
        val totalSinceStart = max(0f, c - b).toLong()
        val diff = totalSinceStart - sessionSteps
        if (diff <= 0L) return 0L
        return min(20L, diff) // clamp
    }

    override fun onSensorChanged(event: SensorEvent) {
        if (event.sensor.type == Sensor.TYPE_STEP_COUNTER) {
            val v = event.values[0]
            if (baseline == null) baseline = v
            lastCounter = v
        }
    }
    override fun onAccuracyChanged(sensor: Sensor?, accuracy: Int) {}
}
// [WC-10.3-RECONCILER-END]
