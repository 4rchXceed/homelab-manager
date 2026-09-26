SELECT last_sync, sync_time, agent.id_str, sync_agent.id_str, sync_agent_id, sync_storage_id_str
FROM service
LEFT OUTER JOIN agent ON service.agent_id = agent.id
LEFT OUTER JOIN agent AS sync_agent ON service.sync_agent_id = sync_agent.id
WHERE service.id_str = ?;
