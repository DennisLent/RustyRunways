import { useState, useRef, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Trash2, FileText } from "lucide-react";

interface LogEntry {
  id: string;
  timestamp: string;
  type: 'info' | 'success' | 'warning' | 'error';
  message: string;
}

interface GameLoggerProps {
  logs: LogEntry[];
  onClearLogs: () => void;
}

export const GameLogger = ({ logs, onClearLogs }: GameLoggerProps) => {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  const getLogTypeColor = (type: LogEntry['type']) => {
    switch (type) {
      case 'success':
        return 'bg-green-500/20 text-green-400 border-green-500/30';
      case 'warning':
        return 'bg-aviation-amber/20 text-aviation-amber border-aviation-amber/30';
      case 'error':
        return 'bg-red-500/20 text-red-400 border-red-500/30';
      default:
        return 'bg-aviation-blue/20 text-aviation-blue border-aviation-blue/30';
    }
  };

  const getLogTypeIcon = (type: LogEntry['type']) => {
    switch (type) {
      case 'success':
        return '✓';
      case 'warning':
        return '⚠';
      case 'error':
        return '✗';
      default:
        return 'ℹ';
    }
  };

  return (
    <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-aviation-blue flex items-center gap-2">
            <FileText className="w-5 h-5" />
            Game Events
          </CardTitle>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="bg-aviation-blue/10 border-aviation-blue/30">
              {logs.length} events
            </Badge>
            <Button variant="ghost" size="sm" onClick={onClearLogs}>
              <Trash2 className="w-4 h-4" />
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-32" ref={scrollRef}>
          <div className="space-y-2">
            {logs.length === 0 ? (
              <div className="text-muted-foreground text-sm text-center py-4">
                No events logged yet
              </div>
            ) : (
              logs.map((log) => (
                <div
                  key={log.id}
                  className="flex items-start gap-3 text-sm"
                >
                  <Badge
                    variant="outline"
                    className={`text-xs shrink-0 ${getLogTypeColor(log.type)}`}
                  >
                    {getLogTypeIcon(log.type)}
                  </Badge>
                  <div className="flex-1 min-w-0">
                    <div className="text-foreground">{log.message}</div>
                    <div className="text-muted-foreground text-xs">
                      {log.timestamp}
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
};