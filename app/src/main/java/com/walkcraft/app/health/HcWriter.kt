// [WC-10.3-HCWRITER-BEGIN]
package com.walkcraft.app.health

import android.content.Context
import androidx.health.connect.client.HealthConnectClient
import androidx.health.connect.client.PermissionController
import androidx.health.connect.client.records.ExerciseSessionRecord
import androidx.health.connect.client.records.StepsRecord
import androidx.health.connect.client.request.ReadRecordsRequest
import androidx.health.connect.client.request.WriteRecordsRequest
import androidx.health.connect.client.time.TimeRangeFilter
import java.time.Instant
import java.time.ZoneId

object HcWriter {
    val writePermissions = setOf(
        PermissionController.createWritePermission(ExerciseSessionRecord::class),
        PermissionController.createWritePermission(StepsRecord::class),
    )

    data class LocalSessionSnapshot(
        val startMs: Long,
        val endMs: Long,
        val steps: Long,
        val note: String? = null
    )
    data class MinuteSteps(val epochMinute: Long, val steps: Int)

    private fun client(ctx: Context) = HealthConnectClient.getOrCreate(ctx)

    suspend fun write(
        ctx: Context,
        local: LocalSessionSnapshot,
        minutes: List<MinuteSteps>,
        writeStepsIfMissing: Boolean
    ): Result<Unit> = runCatching {
        val c = client(ctx)
        val start = Instant.ofEpochMilli(local.startMs)
        val end = Instant.ofEpochMilli(local.endMs)
        val startOff = ZoneId.systemDefault().rules.getOffset(start)
        val endOff = ZoneId.systemDefault().rules.getOffset(end)

        val records = mutableListOf<androidx.health.connect.client.records.Record>()
        records += ExerciseSessionRecord(
            startTime = start, startZoneOffset = startOff,
            endTime = end,   endZoneOffset = endOff,
            exerciseType = ExerciseSessionRecord.ExerciseType.WALKING,
            title = "WalkCraft",
            notes = local.note.orEmpty()
        )

        if (writeStepsIfMissing) {
            val existing = c.readRecords(
                ReadRecordsRequest(
                    StepsRecord::class,
                    timeRangeFilter = TimeRangeFilter.between(start, end)
                )
            ).records
            if (existing.isEmpty()) {
                minutes.forEach { m ->
                    val s = Instant.ofEpochSecond(m.epochMinute * 60)
                    val e = s.plusSeconds(60)
                    records += StepsRecord(
                        count = m.steps,
                        startTime = s, startZoneOffset = startOff,
                        endTime = e,   endZoneOffset = endOff
                    )
                }
            }
        }

        c.insertRecords(WriteRecordsRequest(records))
    }
}
// [WC-10.3-HCWRITER-END]
