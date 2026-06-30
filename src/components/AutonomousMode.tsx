import { useEffect, useState, useCallback } from "react";
import {
  startAutonomousMode,
  stopAutonomousMode,
  isAutonomousRunning,
  getNotifications,
  getUnreadCount,
  markAllNotificationsRead,
  analyzeTickerForAlerts,
  Alert,
  Notification,
} from "../lib/tauri-bridge";
import { useWatchlistStore } from "../stores/useWatchlistStore";

type SubTab = "status" | "alerts" | "notifications" | "analyze";

const severityColor: Record<string, string> = {
  Critical: "text-accent-red",
  High: "text-accent-red",
  Medium: "text-accent-yellow",
  Low: "text-accent-blue",
};

const severityBg: Record<string, string> = {
  Critical: "bg-accent-red/15 border-accent-red/40",
  High: "bg-accent-red/10 border-accent-red/30",
  Medium: "bg-accent-yellow/10 border-accent-yellow/30",
  Low: "bg-accent-blue/10 border-accent-blue/30",
};

function AlertCard({ alert }: { alert: Alert }) {
  return (
    <div
      className={`rounded-lg p-3 border ${severityBg[alert.severity] || "bg-bg-secondary border-border"}`}
    >
      <div className="flex items-center justify-between mb-1">
        <span className="text-fg-primary text-sm font-semibold">
          {alert.ticker}
        </span>
        <span className={`text-xs font-bold ${severityColor[alert.severity] || "text-fg-muted"}`}>
          {alert.severity}
        </span>
      </div>
      <p className="text-fg-primary text-sm font-medium mb-1">{alert.title}</p>
      <p className="text-fg-secondary text-xs">{alert.message}</p>
      <div className="flex items-center gap-3 mt-2">
        <span className="text-fg-muted text-xs">
          {new Date(alert.created_at * 1000).toLocaleString()}
        </span>
        {alert.value !== null && (
          <span className="text-fg-muted text-xs font-mono">
            Value: {alert.value.toFixed(2)}
          </span>
        )}
      </div>
    </div>
  );
}

function NotificationCard({
  notification,
  onRead,
}: {
  notification: Notification;
  onRead: (id: string) => void;
}) {
  return (
    <div
      className={`rounded-lg p-3 border cursor-pointer transition-colors ${
        notification.read
          ? "bg-bg-secondary border-border opacity-60"
          : "bg-bg-secondary border-accent-purple/30 hover:border-accent-purple/50"
      }`}
      onClick={() => !notification.read && onRead(notification.id)}
    >
      <div className="flex items-center justify-between mb-1">
        <span className="text-fg-primary text-sm font-semibold">
          {notification.ticker}
        </span>
        {!notification.read && (
          <span className="w-2 h-2 rounded-full bg-accent-purple" />
        )}
      </div>
      <p className="text-fg-primary text-sm font-medium mb-1">
        {notification.title}
      </p>
      <p className="text-fg-secondary text-xs">{notification.message}</p>
      <span className="text-fg-muted text-xs mt-1 block">
        {new Date(notification.created_at * 1000).toLocaleString()}
      </span>
    </div>
  );
}

export function AutonomousMode() {
  const { items } = useWatchlistStore();
  const [subTab, setSubTab] = useState<SubTab>("status");
  const [running, setRunning] = useState(false);
  const [intervalSeconds, setIntervalSeconds] = useState(300);
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const [analyzeTicker, setAnalyzeTicker] = useState("");
  const [analyzeLoading, setAnalyzeLoading] = useState(false);
  const [analyzeResults, setAnalyzeResults] = useState<Alert[]>([]);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  const refreshNotifications = useCallback(async () => {
    try {
      const notifs = await getNotifications(100, false);
      setNotifications(notifs);
      const count = await getUnreadCount();
      setUnreadCount(count);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => {
    isAutonomousRunning().then(setRunning);
    refreshNotifications();
  }, [refreshNotifications]);

  const handleStart = async () => {
    try {
      await startAutonomousMode(intervalSeconds, true);
      setRunning(true);
      setStatusMsg("Autonomous mode started");
    } catch (e) {
      setStatusMsg(String(e));
    }
  };

  const handleStop = async () => {
    try {
      await stopAutonomousMode();
      setRunning(false);
      setStatusMsg("Autonomous mode stopped");
    } catch (e) {
      setStatusMsg(String(e));
    }
  };

  const handleMarkAllRead = async () => {
    await markAllNotificationsRead();
    refreshNotifications();
  };

  const handleAnalyze = async () => {
    if (!analyzeTicker.trim()) return;
    setAnalyzeLoading(true);
    setAnalyzeResults([]);
    try {
      const results = await analyzeTickerForAlerts(analyzeTicker.toUpperCase().trim(), false);
      setAnalyzeResults(results);
    } catch (e) {
      setStatusMsg(String(e));
    } finally {
      setAnalyzeLoading(false);
    }
  };

  const subTabs: { key: SubTab; label: string; badge?: number }[] = [
    { key: "status", label: "Status" },
    { key: "analyze", label: "Quick Scan" },
    { key: "alerts", label: "Alerts" },
    { key: "notifications", label: "Notifications", badge: unreadCount > 0 ? unreadCount : undefined },
  ];

  return (
    <div className="flex flex-col h-full">
      <div className="flex border-b border-border">
        {subTabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setSubTab(tab.key)}
            className={`flex-1 px-2 py-2 text-xs font-semibold transition-colors relative ${
              subTab === tab.key
                ? "text-accent-purple border-b-2 border-accent-purple"
                : "text-fg-secondary hover:text-fg-primary"
            }`}
          >
            {tab.label}
            {tab.badge !== undefined && (
              <span className="ml-1 px-1.5 py-0.5 bg-accent-red text-bg-primary text-[10px] rounded-full font-bold">
                {tab.badge}
              </span>
            )}
          </button>
        ))}
      </div>

      <main className="flex-1 overflow-y-auto p-3 flex flex-col gap-3">
        {statusMsg && (
          <div
            className="p-2 rounded-lg text-sm bg-accent-purple/10 border border-accent-purple/30 text-accent-purple cursor-pointer"
            onClick={() => setStatusMsg(null)}
          >
            {statusMsg}
          </div>
        )}

        {subTab === "status" && (
          <div className="flex flex-col gap-4">
            <div className="bg-bg-secondary rounded-lg p-4 border border-border">
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-fg-primary text-sm font-semibold">
                  Autonomous Mode
                </h3>
                <span
                  className={`text-xs font-bold ${
                    running ? "text-accent-green" : "text-fg-muted"
                  }`}
                >
                  {running ? "ACTIVE" : "INACTIVE"}
                </span>
              </div>

              <div className="flex items-center gap-3 mb-4">
                <div
                  className={`w-3 h-3 rounded-full ${
                    running
                      ? "bg-accent-green animate-pulse"
                      : "bg-fg-muted"
                  }`}
                />
                <span className="text-fg-secondary text-sm">
                  {running
                    ? "Monitoring watchlist for signals..."
                    : "Not monitoring"}
                </span>
              </div>

              <div className="flex items-center gap-2 mb-4">
                <span className="text-fg-secondary text-xs">Scan interval:</span>
                <input
                  type="number"
                  min={30}
                  max={3600}
                  value={intervalSeconds}
                  onChange={(e) =>
                    setIntervalSeconds(parseInt(e.target.value) || 300)
                  }
                  disabled={running}
                  className="w-20 bg-bg-tertiary text-fg-primary border border-border rounded px-2 py-1 text-sm font-mono disabled:opacity-50"
                />
                <span className="text-fg-muted text-xs">sec</span>
              </div>

              {running ? (
                <button
                  onClick={handleStop}
                  className="w-full px-4 py-2 bg-accent-red text-bg-primary text-sm font-semibold rounded-lg hover:bg-accent-red/80 transition-colors"
                >
                  Stop Autonomous Mode
                </button>
              ) : (
                <button
                  onClick={handleStart}
                  disabled={items.length === 0}
                  className="w-full px-4 py-2 bg-accent-green text-bg-primary text-sm font-semibold rounded-lg hover:bg-accent-green/80 disabled:opacity-50 transition-colors"
                >
                  Start Autonomous Mode
                </button>
              )}
            </div>

            <div className="bg-bg-secondary rounded-lg p-4 border border-border">
              <h3 className="text-accent-purple text-xs font-semibold mb-2 uppercase tracking-wide">
                What it does
              </h3>
              <ul className="text-fg-secondary text-xs space-y-2">
                <li className="flex items-start gap-2">
                  <span className="text-accent-green mt-0.5">1.</span>
                  <span>
                    Continuously scans your watchlist at the configured interval
                  </span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-accent-green mt-0.5">2.</span>
                  <span>
                    Evaluates RSI, MACD, Bollinger Bands, Volume, Support/Resistance
                  </span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-accent-green mt-0.5">3.</span>
                  <span>
                    Generates alerts for oversold, overbought, breakouts, volume spikes
                  </span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-accent-green mt-0.5">4.</span>
                  <span>
                    Stores notifications for review in the Notifications tab
                  </span>
                </li>
              </ul>
            </div>

            <div className="bg-bg-secondary rounded-lg p-3 border border-border">
              <div className="flex items-center justify-between">
                <span className="text-fg-secondary text-xs">
                  Watchlist size
                </span>
                <span className="text-fg-primary text-sm font-mono">
                  {items.length} tickers
                </span>
              </div>
              <div className="flex items-center justify-between mt-1">
                <span className="text-fg-secondary text-xs">
                  Unread notifications
                </span>
                <span className="text-fg-primary text-sm font-mono">
                  {unreadCount}
                </span>
              </div>
            </div>
          </div>
        )}

        {subTab === "analyze" && (
          <div className="flex flex-col gap-4">
            <div className="bg-bg-secondary rounded-lg p-3 border border-border">
              <h3 className="text-accent-purple text-xs font-semibold mb-2 uppercase tracking-wide">
                Quick Signal Scan
              </h3>
              <p className="text-fg-secondary text-xs mb-3">
                Analyze a single ticker for technical signals without full
                synthesis.
              </p>
              <div className="flex gap-2">
                <input
                  value={analyzeTicker}
                  onChange={(e) => setAnalyzeTicker(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleAnalyze()}
                  placeholder="e.g. AAPL"
                  className="flex-1 bg-bg-tertiary text-fg-primary border border-border rounded px-3 py-1.5 text-sm font-mono uppercase"
                />
                <button
                  onClick={handleAnalyze}
                  disabled={analyzeLoading || !analyzeTicker.trim()}
                  className="px-3 py-1.5 bg-accent-purple text-bg-primary text-sm font-semibold rounded hover:bg-accent-purple/80 disabled:opacity-50 transition-colors"
                >
                  {analyzeLoading ? "Scanning..." : "Scan"}
                </button>
              </div>
            </div>

            {analyzeResults.length > 0 && (
              <div className="flex flex-col gap-2">
                <h3 className="text-fg-secondary text-xs font-semibold">
                  Signals Found ({analyzeResults.length})
                </h3>
                {analyzeResults.map((alert) => (
                  <AlertCard key={alert.id} alert={alert} />
                ))}
              </div>
            )}

            {analyzeResults.length === 0 && !analyzeLoading && analyzeTicker && (
              <div className="text-center py-8">
                <p className="text-fg-muted text-sm">
                  No signals detected for {analyzeTicker.toUpperCase()}
                </p>
              </div>
            )}
          </div>
        )}

        {subTab === "alerts" && (
          <div className="flex flex-col gap-2">
            <p className="text-fg-secondary text-xs">
              All alerts generated by autonomous mode and quick scans.
            </p>
            {notifications.length === 0 ? (
              <div className="text-center py-8">
                <p className="text-fg-muted text-sm">No alerts yet</p>
                <p className="text-fg-muted text-xs mt-1">
                  Start autonomous mode or run a quick scan
                </p>
              </div>
            ) : (
              notifications.map((n) => (
                <NotificationCard
                  key={n.id}
                  notification={n}
                  onRead={async (id) => {
                    const { markNotificationRead } = await import(
                      "../lib/tauri-bridge"
                    );
                    await markNotificationRead(id);
                    refreshNotifications();
                  }}
                />
              ))
            )}
          </div>
        )}

        {subTab === "notifications" && (
          <div className="flex flex-col gap-3">
            <div className="flex items-center justify-between">
              <h3 className="text-fg-secondary text-xs font-semibold">
                Notifications ({unreadCount} unread)
              </h3>
              {unreadCount > 0 && (
                <button
                  onClick={handleMarkAllRead}
                  className="text-accent-purple text-xs hover:underline"
                >
                  Mark all read
                </button>
              )}
            </div>
            {notifications.length === 0 ? (
              <div className="text-center py-8">
                <p className="text-fg-muted text-sm">No notifications</p>
              </div>
            ) : (
              notifications.map((n) => (
                <NotificationCard
                  key={n.id}
                  notification={n}
                  onRead={async (id) => {
                    const { markNotificationRead } = await import(
                      "../lib/tauri-bridge"
                    );
                    await markNotificationRead(id);
                    refreshNotifications();
                  }}
                />
              ))
            )}
          </div>
        )}
      </main>
    </div>
  );
}
