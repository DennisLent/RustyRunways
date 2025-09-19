import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { WorldMap } from "@/components/WorldMap";
import { GameLogger } from "@/components/GameLogger";
import { AirplaneDetailScreen } from "@/components/AirplaneDetailScreen";
import { AirportDetailScreen } from "@/components/AirportDetailScreen";
import { AirplanePurchaseScreen } from "@/components/AirplanePurchaseScreen";
import { 
  Plane, 
  Building2, 
  Package, 
  DollarSign, 
  Clock, 
  Save, 
  FolderOpen, 
  Settings,
  LogOut,
  Play,
  Pause,
  BarChart3,
  ShoppingCart,
  Users
} from "lucide-react";
import { observe, advance as apiAdvance, saveGame as apiSave, listSaves as apiListSaves, loadGame as apiLoadGame } from "@/api/game";
import type { Observation } from "@/api/game";

interface GameScreenProps {
  onMainMenu: () => void;
}

interface LogEntry {
  id: string;
  timestamp: string;
  type: 'info' | 'success' | 'warning' | 'error';
  message: string;
}

type ScreenMode = 'main' | 'airplane' | 'airport' | 'purchase';

interface Airport {
  id: string;
  name: string;
  code: string;
  x: number;
  y: number;
  hasOrders: boolean;
  orderCount: number;
}

interface Airplane {
  id: string;
  model: string;
  x: number;
  y: number;
  status: 'parked' | 'en-route' | 'loading';
  destination?: string;
}

export const GameScreen = ({ onMainMenu }: GameScreenProps) => {
  const [screenMode, setScreenMode] = useState<ScreenMode>('main');
  const [selectedAirplaneId, setSelectedAirplaneId] = useState<string>("");
  const [selectedAirportId, setSelectedAirportId] = useState<string>("");
  const [cash, setCash] = useState<number>(0);
  const [timeStr, setTimeStr] = useState<string>("0h");
  type ObsAirport = Observation["airports"][number];
  type ObsPlane = Observation["planes"][number];
  const [airports, setAirports] = useState<ObsAirport[]>([]);
  const [planes, setPlanes] = useState<ObsPlane[]>([]);
  const timerRef = useRef<number | null>(null);
  const [saveOpen, setSaveOpen] = useState(false);
  const [loadOpen, setLoadOpen] = useState(false);
  const [saveName, setSaveName] = useState("");
  const [availableSaves, setAvailableSaves] = useState<string[]>([]);
  
  const [logs, setLogs] = useState<LogEntry[]>([]);

  async function refresh() {
    const obs = await observe();
    setCash(obs.cash);
    setTimeStr(`${Math.floor(obs.time / 24)}d ${obs.time % 24}h`);
    // keep arrays for possible future use
    setAirports(obs.airports);
    setPlanes(obs.planes);
  }

  useEffect(() => {
    refresh();
  }, []);

  const addLog = (type: LogEntry['type'], message: string) => {
    const now = new Date();
    const timestamp = now.toLocaleTimeString();
    const newLog: LogEntry = {
      id: Date.now().toString(),
      timestamp,
      type,
      message
    };
    setLogs(prev => [...prev, newLog]);
  };

  const clearLogs = () => {
    setLogs([]);
  };

  const handleAirplaneClick = (airplane: Airplane) => {
    setSelectedAirplaneId(airplane.id);
    // Defer screen change to ensure selectedAirplaneId is committed
    setTimeout(() => setScreenMode('airplane'), 0);
    addLog('info', `Viewing details for ${airplane.model} ${airplane.id}`);
  };

  const handleAirportClick = (airport: Airport) => {
    setSelectedAirportId(airport.id);
    setScreenMode('airport');
    addLog('info', `Viewing ${airport.name} (${airport.code})`);
  };

  const handleBackToMain = () => {
    setScreenMode('main');
    setSelectedAirplaneId("");
    setSelectedAirportId("");
  };

  const handlePlaneSold = async (planeId: string, refund: number) => {
    addLog('success', `Sold plane ${planeId} for $${refund.toFixed(2)}`);
    await refresh();
    handleBackToMain();
  };

  const handleAdvanceTime = async () => {
    const obs = await apiAdvance(1);
    setCash(obs.cash);
    setTimeStr(`${Math.floor(obs.time / 24)}d ${obs.time % 24}h`);
    setAirports(obs.airports);
    setPlanes(obs.planes);
    addLog('info', 'Advanced by 1 hour');
  };

  const handlePlay = () => {
    if (timerRef.current) return;
    timerRef.current = window.setInterval(() => {
      handleAdvanceTime();
    }, 500);
  };

  const handlePause = () => {
    if (timerRef.current) {
      window.clearInterval(timerRef.current);
      timerRef.current = null;
    }
  };

  const handleSave = async () => {
    setSaveOpen(true);
  };

  const handleDispatch = (airplaneId: string, destination: string) => {
    addLog('success', `${airplaneId} dispatched to ${destination}`);
  };

  const handleLoadOrder = (airplaneId: string, orderId: string) => {
    addLog('success', `Order ${orderId} loaded onto ${airplaneId}`);
  };

  const handleUnloadOrder = (airplaneId: string, orderId: string) => {
    addLog('info', `Order ${orderId} unloaded from ${airplaneId}`);
  };

  const handleRefuel = (airplaneId: string) => {
    addLog('info', `${airplaneId} refueling initiated`);
  };

  const handleMaintenance = (airplaneId: string) => {
    addLog('warning', `${airplaneId} scheduled for maintenance`);
  };

  const handlePurchase = (aircraftId: string, airportId: string) => {
    addLog('success', `New aircraft ${aircraftId} purchased at ${airportId}`);
    setScreenMode('main');
  };

  const gameStats = {
    cash: cash,
    time: timeStr,
    planes: planes.length,
    activeOrders: airports.reduce((a, b) => a + (b.num_orders || 0), 0),
    completedDeliveries: 0,
    totalRevenue: 0
  };

  // Render different screens based on mode
  if (screenMode === 'airplane') {
    if (!selectedAirplaneId) {
      return (
        <div className="min-h-screen bg-gradient-control p-4 flex items-center justify-center text-muted-foreground">
          Loading aircraft...
        </div>
      );
    }
    return (
      <AirplaneDetailScreen
        airplaneId={selectedAirplaneId}
        onBack={handleBackToMain}
        airportsData={airports.map((a) => ({ id: a.id, name: a.name }))}
        onSold={handlePlaneSold}
      />
    );
  }

  if (screenMode === 'airport') {
    return (
      <AirportDetailScreen
        airportId={selectedAirportId}
        onBack={handleBackToMain}
        airportsData={airports}
        planesData={planes}
        onAirplaneClick={(airplaneId: string) => {
          setSelectedAirplaneId(airplaneId);
          setScreenMode('airplane');
        }}
      />
    );
  }

  if (screenMode === 'purchase') {
    return (
      <AirplanePurchaseScreen
        onBack={handleBackToMain}
        onPurchase={async (_aircraftId: string, _airportId: string) => {
          // After purchase, refresh observation and go back
          await refresh();
          setScreenMode('main');
        }}
        playerCash={gameStats.cash}
        airportsData={airports.map((a) => ({ id: a.id, name: a.name }))}
      />
    );
  }

  return (
    <div className="min-h-screen bg-gradient-control p-4">
      <div className="max-w-7xl mx-auto space-y-4">
        
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <div className="p-2 rounded-lg bg-aviation-blue/20 border border-aviation-blue/30">
                <Plane className="w-5 h-5 text-aviation-blue" />
              </div>
              <h1 className="text-2xl font-bold text-foreground">Rusty Runways</h1>
            </div>
            
            {/* Game Stats */}
            <div className="flex items-center gap-4 text-sm">
              <Badge variant="outline" className="bg-aviation-blue/10 border-aviation-blue/30">
                <DollarSign className="w-3 h-3 mr-1" />
                ${gameStats.cash.toLocaleString()}
              </Badge>
              <Badge variant="outline" className="bg-aviation-amber/10 border-aviation-amber/30">
                <Clock className="w-3 h-3 mr-1" />
                {gameStats.time}
              </Badge>
              <Badge variant="outline" className="bg-aviation-radar/10 border-aviation-radar/30">
                <Plane className="w-3 h-3 mr-1" />
                {gameStats.planes} Planes
              </Badge>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Button variant="control" size="sm" onClick={handleSave}>
              <Save className="w-4 h-4 mr-1" />
              Save
            </Button>
            <Button variant="control" size="sm" onClick={async () => { setLoadOpen(true); setAvailableSaves(await apiListSaves()); }}>
              <FolderOpen className="w-4 h-4 mr-1" />
              Load
            </Button>
            <Button variant="control" size="sm">
              <Settings className="w-4 h-4" />
            </Button>
            <Button variant="warning" size="sm" onClick={onMainMenu}>
              <LogOut className="w-4 h-4 mr-1" />
              Exit
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-12 gap-4">
          
          {/* Main Map Area */}
          <div className="col-span-8 space-y-4">
            
            {/* Quick Actions */}
            <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
              <CardContent className="p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Button variant="runway" size="sm" onClick={handlePlay}>
                      <Play className="w-3 h-3 mr-1" />
                      Play
                    </Button>
                    <Button variant="control" size="sm" onClick={handlePause}>
                      <Pause className="w-3 h-3 mr-1" />
                      Pause
                    </Button>
                    <Button 
                      variant="control" 
                      size="sm" 
                      onClick={() => setScreenMode('purchase')}
                    >
                      <ShoppingCart className="w-3 h-3 mr-1" />
                      Buy Aircraft
                    </Button>
                  </div>
                  
                  <div className="flex items-center gap-2 text-sm">
                    <Badge variant="outline" className="bg-aviation-blue/10 border-aviation-blue/30">
                      <Building2 className="w-3 h-3 mr-1" />
                      {airports.length} Airports
                    </Badge>
                    <Badge variant="outline" className="bg-aviation-amber/10 border-aviation-amber/30">
                      <Package className="w-3 h-3 mr-1" />
                      {gameStats.activeOrders} Active Orders
                    </Badge>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* World Map */}
            <div className="h-96">
              <WorldMap 
                onAirportClick={handleAirportClick}
                onAirplaneClick={handleAirplaneClick}
                airportsData={airports.map((a) => ({
                  id: String(a.id),
                  name: a.name,
                  code: String(a.id),
                  x: a.x,
                  y: a.y,
                  hasOrders: (a.num_orders ?? 0) > 0,
                  orderCount: a.num_orders ?? 0,
                }))}
                airplanesData={planes.map((p) => ({
                  id: String(p.id),
                  model: p.model,
                  x: p.x,
                  y: p.y,
                  status: p.status.includes('InTransit') ? 'en-route' : (p.status.includes('Loading') ? 'loading' : 'parked'),
                  destination: p.destination != null ? String(p.destination) : undefined,
                }))}
              />
            </div>

            {/* Game Logger */}
            <GameLogger 
              logs={logs}
              onClearLogs={clearLogs}
            />
          </div>

          {/* Side Panel */}
          <div className="col-span-4">
            <Tabs defaultValue="overview" className="space-y-4">
              <TabsList className="grid w-full grid-cols-3 bg-secondary/50">
                <TabsTrigger value="overview">Overview</TabsTrigger>
                <TabsTrigger value="fleet">Fleet</TabsTrigger>
                <TabsTrigger value="airports">Airports</TabsTrigger>
              </TabsList>

              <TabsContent value="overview" className="space-y-4">
                
                {/* Game Stats */}
                <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
                  <CardHeader className="pb-3">
                    <CardTitle className="text-aviation-blue text-sm">Game Statistics</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-3 text-sm">
                    <div className="grid grid-cols-2 gap-3">
                      <div className="space-y-1">
                        <div className="text-muted-foreground">Revenue</div>
                        <div className="font-semibold text-aviation-radar">
                          ${gameStats.totalRevenue.toLocaleString()}
                        </div>
                      </div>
                      <div className="space-y-1">
                        <div className="text-muted-foreground">Deliveries</div>
                        <div className="font-semibold">{gameStats.completedDeliveries}</div>
                      </div>
                      <div className="space-y-1">
                        <div className="text-muted-foreground">Active Orders</div>
                        <div className="font-semibold text-aviation-amber">{gameStats.activeOrders}</div>
                      </div>
                      <div className="space-y-1">
                        <div className="text-muted-foreground">Fleet Size</div>
                        <div className="font-semibold">{gameStats.planes}</div>
                      </div>
                    </div>
                  </CardContent>
                </Card>

                {/* Quick Commands */}
                <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
                  <CardHeader className="pb-3">
                    <CardTitle className="text-aviation-blue text-sm">Quick Commands</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-2">
                    <Button 
                      variant="ghost" 
                      size="sm" 
                      className="w-full justify-start h-8"
                      onClick={() => addLog('info', 'Viewing game statistics')}
                    >
                      <BarChart3 className="w-3 h-3 mr-2" />
                      View Statistics
                    </Button>
                    <Button 
                      variant="ghost" 
                      size="sm" 
                      className="w-full justify-start h-8"
                      onClick={() => addLog('info', `Current time: ${gameStats.time}`)}
                    >
                      <Clock className="w-3 h-3 mr-2" />
                      Current Time
                    </Button>
                    <Button 
                      variant="ghost" 
                      size="sm" 
                      className="w-full justify-start h-8"
                      onClick={() => addLog('info', 'Checked pending orders')}
                    >
                      <Package className="w-3 h-3 mr-2" />
                      Pending Orders
                    </Button>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="fleet" className="space-y-4">
                <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
                  <CardHeader className="pb-3">
                    <CardTitle className="text-aviation-blue text-sm">Active Fleet</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-3 text-sm">
                    <div className="space-y-2">
                      {planes.length === 0 && (
                        <div className="text-muted-foreground text-xs">No aircraft owned yet.</div>
                      )}
                      {planes.map((p) => {
                        const fuelPct = p.fuel?.capacity > 0 ? Math.round((p.fuel.current / p.fuel.capacity) * 100) : 0;
                        const status: 'parked' | 'en-route' | 'loading' = p.status.includes('InTransit')
                          ? 'en-route'
                          : (p.status.includes('Loading') ? 'loading' : 'parked');
                        const atAirport = airports.find((a) => !p.status.includes('InTransit') && a.x === p.x && a.y === p.y);
                        const destAirport = p.destination != null ? airports.find((a) => a.id === p.destination) : undefined;
                        const locLabel = atAirport ? String(atAirport.name) : (destAirport ? `→ ${destAirport.name}` : '');
                        return (
                          <div key={p.id} className="border border-aviation-blue/20 rounded-lg p-3 bg-secondary/20">
                            <div className="font-semibold">{p.model}</div>
                            <div className="text-muted-foreground text-xs">
                              {locLabel ? `Location: ${locLabel} • ` : ''}Status: {status.replace('-', ' ')} • Fuel: {fuelPct}%
                            </div>
                            <div className="flex gap-1 mt-2">
                              <Button 
                                variant="ghost" 
                                size="sm" 
                                className="h-6 text-xs"
                                onClick={() => handleAirplaneClick({ id: String(p.id), model: p.model, x: p.x, y: p.y, status, destination: p.destination != null ? String(p.destination) : undefined })}
                              >
                                <Users className="w-3 h-3 mr-1" />
                                Manage
                              </Button>
                            </div>
                          </div>
                        );
                      })}
                    </div>

                    <Button 
                      variant="control" 
                      size="sm" 
                      className="w-full"
                      onClick={() => setScreenMode('purchase')}
                    >
                      <ShoppingCart className="w-3 h-3 mr-1" />
                      Buy New Plane
                    </Button>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="airports" className="space-y-4">
                <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
                  <CardHeader className="pb-3">
                    <CardTitle className="text-aviation-blue text-sm">Airports</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-2 text-sm">
                    {airports.length === 0 && (
                      <div className="text-muted-foreground text-xs">No airports available.</div>
                    )}
                    {airports.map((a) => {
                      const aircraft = planes.filter((p) => !p.status.includes('InTransit') && p.x === a.x && p.y === a.y).length;
                      return (
                        <div 
                          key={a.id} 
                          className="border border-aviation-blue/20 rounded-lg p-3 bg-secondary/20 hover:bg-secondary/30 transition-colors cursor-pointer"
                          onClick={() => handleAirportClick({ 
                            id: String(a.id), 
                            name: a.name, 
                            code: String(a.id), 
                            x: a.x, 
                            y: a.y, 
                            hasOrders: (a.num_orders ?? 0) > 0, 
                            orderCount: a.num_orders ?? 0 
                          })}
                        >
                          <div className="flex justify-between items-start">
                            <div>
                              <div className="font-semibold">{a.name}</div>
                              <div className="text-muted-foreground text-xs">ID: {a.id}</div>
                            </div>
                            <div className="text-right text-xs">
                              <div className="text-aviation-amber">{a.num_orders ?? 0} orders</div>
                              <div className="text-muted-foreground">{aircraft} aircraft</div>
                            </div>
                          </div>
                        </div>
                      );
                    })}
                  </CardContent>
                </Card>
              </TabsContent>
            </Tabs>
          </div>
        </div>
      </div>

      {saveOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center">
          <div className="bg-card p-4 rounded border border-aviation-blue/20 w-96">
            <div className="text-lg font-semibold mb-2">Save Game</div>
            <input
              className="w-full bg-secondary/50 border border-aviation-blue/20 rounded px-2 py-1 mb-3"
              placeholder="Save name"
              value={saveName}
              onChange={e => setSaveName(e.target.value)}
            />
            <div className="flex justify-end gap-2">
              <Button variant="ghost" size="sm" onClick={() => setSaveOpen(false)}>Cancel</Button>
              <Button variant="runway" size="sm" onClick={async () => { if (!saveName) return; await apiSave(saveName); addLog('success', `Saved game: ${saveName}`); setSaveOpen(false); setSaveName(''); }}>Save</Button>
            </div>
          </div>
        </div>
      )}

      {loadOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center">
          <div className="bg-card p-4 rounded border border-aviation-blue/20 w-[28rem]">
            <div className="text-lg font-semibold mb-2">Load Game</div>
            <div className="max-h-64 overflow-auto border border-aviation-blue/20 rounded mb-3">
              {availableSaves.length === 0 ? (
                <div className="p-3 text-muted-foreground text-sm">No save files found</div>
              ) : (
                availableSaves.map(name => (
                  <div key={name} className="px-3 py-2 hover:bg-secondary/30 cursor-pointer flex justify-between items-center"
                    onClick={async () => { await apiLoadGame(name); await refresh(); setLoadOpen(false); addLog('success', `Loaded game: ${name}`); }}>
                    <span>{name}</span>
                    <Button variant="ghost" size="sm">Load</Button>
                  </div>
                ))
              )}
            </div>
            <div className="flex justify-end">
              <Button variant="ghost" size="sm" onClick={() => setLoadOpen(false)}>Close</Button>
            </div>
          </div>
        </div>
      )}

    </div>
  );
};

// Simple inline dialogs for Save/Load
