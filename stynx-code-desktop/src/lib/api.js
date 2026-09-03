import { invoke } from "@tauri-apps/api/core";

export const initSession = (workspacePath, provider) =>
  invoke("init_session", { workspacePath, provider });
export const getSessionInfo = () => invoke("get_session_info");
export const sendMessage = (text, images) => invoke("send_message", { text, images });
export const cancel = () => invoke("cancel");
export const setModel = (model) => invoke("set_model", { model });
export const setMode = (mode) => invoke("set_mode", { mode });
export const setThinking = (enabled) => invoke("set_thinking", { enabled });
export const setProviderKey = (envName, value) =>
  invoke("set_provider_key", { envName, value });
export const listSessions = () => invoke("list_sessions");
export const loadSession = (id) => invoke("load_session", { id });
export const newSession = () => invoke("new_session");
export const deleteSession = (id) => invoke("delete_session", { id });
export const respondPermission = (id, choice) =>
  invoke("respond_permission", { id, choice });
export const respondAskUser = (id, answer) => invoke("respond_ask_user", { id, answer });
export const respondWorkspaceMessage = (id, reply) =>
  invoke("respond_workspace_message", { id, reply });
export const readFile = (path) => invoke("read_file", { path });
export const fetchReference = (url) => invoke("fetch_reference", { url });
export const listProjectFiles = (root) => invoke("list_project_files", { root });
