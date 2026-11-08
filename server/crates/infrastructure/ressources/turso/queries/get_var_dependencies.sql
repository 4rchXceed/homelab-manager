SELECT service.id_str, service.id
FROM user_var_needs_update
INNER JOIN service ON user_var_needs_update.service_id = service.id
WHERE user_var_needs_update.user_variable_id = ?;
