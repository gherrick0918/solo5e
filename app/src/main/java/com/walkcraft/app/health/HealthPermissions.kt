package com.walkcraft.app.health

object HealthPermissions {
    private val existingReadPerms = emptySet<androidx.health.connect.client.Permission>()

    fun allPermissions(): Set<androidx.health.connect.client.Permission> {
        // [WC-10.3-PERMISSIONS-BEGIN]
        val writePerms = com.walkcraft.app.health.HcWriter.writePermissions
        val allPerms = existingReadPerms + writePerms
        // Request 'allPerms' using your current flow
        // [WC-10.3-PERMISSIONS-END]
        return allPerms
    }
}
