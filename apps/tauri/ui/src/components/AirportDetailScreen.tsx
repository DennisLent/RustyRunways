import { useEffect, useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { 
  ArrowLeft, 
  Building2, 
  Package, 
  Fuel, 
  Plane,
  Filter,
  DollarSign,
  Clock,
  MapPin
} from "lucide-react";
import { airportOrders as apiAirportOrders, loadOrder as apiLoad } from "@/api/game";

interface Order {
  id: string;
  cargoType: string;
  weight: number;
  destination: string;
  deadline: string;
  value: number;
}

interface Airplane {
  id: string;
  model: string;
  status: string;
  fuel: number;
  cargoLoad: number;
}

interface AirportDetailScreenProps {
  airportId: string;
  onBack: () => void;
  onAirplaneClick: (airplaneId: string) => void;
  airportsData: { id: number; name: string; x: number; y: number; fuel_price: number; runway_length: number; num_orders: number }[];
  planesData: { id: number; model: string; x: number; y: number; status: string }[];
}

export const AirportDetailScreen = ({ airportId, onBack, onAirplaneClick, airportsData, planesData }: AirportDetailScreenProps) => {
  const [filterDestination, setFilterDestination] = useState("");
  const [dispatchDest, setDispatchDest] = useState("");
  const [filterWeight, setFilterWeight] = useState("");
  const [filterValue, setFilterValue] = useState("");
  const [filterDeadline, setFilterDeadline] = useState("");
  const [selectedPlaneId, setSelectedPlaneId] = useState<string>("");
  const [orders, setOrders] = useState<Order[]>([]);
  const [selectedOrders, setSelectedOrders] = useState<Record<string, boolean>>({});
  const [planeCapacity, setPlaneCapacity] = useState<{ current: number; capacity: number } | null>(null);
  const [canFlyCache, setCanFlyCache] = useState<Record<number, boolean>>({});
  const [manifest, setManifest] = useState<Order[]>([]);
  const [dispatchInfo, setDispatchInfo] = useState<{ ok: boolean; reason?: string } | null>(null);

  const airportObj = useMemo(() => {
    const idNum = parseInt(airportId, 10);
    return airportsData.find(a => a.id === idNum);
  }, [airportId, airportsData]);

  const airplanesAtAirport: Airplane[] = useMemo(() => {
    if (!airportObj) return [];
    const ax = airportObj.x;
    const ay = airportObj.y;
    return planesData
      .filter(p => !p.status.includes('InTransit') && p.x === ax && p.y === ay)
      .map(p => ({ id: String(p.id), model: p.model, status: p.status, fuel: 0, cargoLoad: 0 }));
  }, [airportObj, planesData]);

  const [planeInfoMap, setPlaneInfoMap] = useState<Record<string, { fuelPct: number; cargoPct: number }>>({});
  useEffect(() => {
    (async () => {
      const map: Record<string, { fuelPct: number; cargoPct: number }> = {};
      for (const plane of airplanesAtAirport) {
        try {
          const info = await (await import('@/api/game')).planeInfo(parseInt(plane.id, 10));
          const fuelPct = info.fuel_capacity > 0 ? Math.round((info.fuel_current / info.fuel_capacity) * 100) : 0;
          const cargoPct = info.payload_capacity > 0 ? Math.round((info.payload_current / info.payload_capacity) * 100) : 0;
          map[plane.id] = { fuelPct, cargoPct };
        } catch (_) { void 0 }
      }
      setPlaneInfoMap(map);
    })();
  }, [airplanesAtAirport.map(p => p.id).join(',')]);

  useEffect(() => {
    (async () => {
      setDispatchInfo(null);
      if (!selectedPlaneId || !dispatchDest) return;
      try {
        const { reachability } = await import('@/api/game');
        const info = await reachability(parseInt(selectedPlaneId, 10), parseInt(dispatchDest, 10));
        setDispatchInfo(info);
      } catch (_) { void 0 }
    })();
  }, [selectedPlaneId, dispatchDest]);

  useEffect(() => {
    async function refresh() {
      const idNum = parseInt(airportId, 10);
      const list = await apiAirportOrders(idNum);
      setOrders(list.map(o => ({
        id: String(o.id),
        cargoType: o.cargo_type || "",
        weight: o.weight,
        destination: String(o.destination_id),
        deadline: String(o.deadline),
        value: o.value,
      })));
    }
    refresh();
  }, [airportId]);

  useEffect(() => {
    async function fetchPlaneInfoAndEligibility() {
      setSelectedOrders({});
      setCanFlyCache({});
      setPlaneCapacity(null);
      if (!selectedPlaneId) return;
      // fetch capacity using planeInfo
      try {
        const idNum = parseInt(selectedPlaneId, 10);
        const info = await (await import('@/api/game')).planeInfo(idNum);
        setPlaneCapacity({ current: info.payload_current, capacity: info.payload_capacity });
        setManifest(info.manifest.map(o => ({
          id: String(o.id),
          cargoType: "",
          weight: o.weight,
          destination: String(o.destination_id),
          deadline: String(o.deadline),
          value: o.value,
        })));
        // compute canFly for unique destinations
        const destSet = new Set<number>(orders.map(o => parseInt(o.destination, 10)));
        const cache: Record<number, boolean> = {};
        for (const d of destSet) {
          try {
            const ok = await (await import('@/api/game')).canFly(idNum, d);
            cache[d] = ok;
          } catch (_) { void 0 }
        }
        setCanFlyCache(cache);
      } catch (_) { void 0 }
    }
    fetchPlaneInfoAndEligibility();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedPlaneId, orders.length]);

  const airport = {
    id: airportId,
    name: airportObj?.name ?? `Airport ${airportId}`,
    code: String(airportObj?.id ?? airportId),
    location: "",
    fuelPrice: airportObj?.fuel_price ?? 0,
    orderCount: orders.length,
    aircraftCount: airplanesAtAirport.length
  };

  const airplanes: Airplane[] = airplanesAtAirport;

  const filteredOrders = orders.filter(order => {
    return (
      (!filterDestination || order.destination.toLowerCase().includes(filterDestination.toLowerCase())) &&
      (!filterWeight || order.weight <= (parseInt(filterWeight) || Infinity)) &&
      (!filterValue || order.value >= (parseInt(filterValue) || 0)) &&
      (!filterDeadline || order.deadline >= filterDeadline)
    );
  });

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'loading':
        return 'bg-aviation-amber/20 text-aviation-amber border-aviation-amber/30';
      case 'parked':
        return 'bg-aviation-blue/20 text-aviation-blue border-aviation-blue/30';
      case 'maintenance':
        return 'bg-red-500/20 text-red-400 border-red-500/30';
      default:
        return 'bg-slate-500/20 text-slate-400 border-slate-500/30';
    }
  };

  return (
    <div className="min-h-screen bg-gradient-control p-4">
      <div className="max-w-7xl mx-auto space-y-4">
        
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Button variant="control" onClick={onBack}>
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back
            </Button>
            <div className="flex items-center gap-2">
              <div className="p-2 rounded-lg bg-aviation-blue/20 border border-aviation-blue/30">
                <Building2 className="w-5 h-5 text-aviation-blue" />
              </div>
              <div>
                <h1 className="text-2xl font-bold text-foreground">{airport.name}</h1>
                <p className="text-muted-foreground">{airport.code} • {airport.location}</p>
              </div>
            </div>
          </div>
          
          <div className="flex items-center gap-4 text-sm">
            <Badge variant="outline" className="bg-aviation-amber/10 border-aviation-amber/30">
              <Fuel className="w-3 h-3 mr-1" />
              Fuel: ${airport.fuelPrice}/gal
            </Badge>
            <Badge variant="outline" className="bg-aviation-blue/10 border-aviation-blue/30">
              <Package className="w-3 h-3 mr-1" />
              {airport.orderCount} Orders
            </Badge>
            <Badge variant="outline" className="bg-aviation-radar/10 border-aviation-radar/30">
              <Plane className="w-3 h-3 mr-1" />
              {airport.aircraftCount} Aircraft
            </Badge>
          </div>
        </div>

        <div className="grid grid-cols-12 gap-4">
          
          {/* Left Panel - Aircraft at Airport */}
          <div className="col-span-4 space-y-4">
            <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
              <CardHeader className="pb-3">
                <CardTitle className="text-aviation-blue">Aircraft at {airport.code}</CardTitle>
              </CardHeader>
              <CardContent>
                <ScrollArea className="h-96">
                  <div className="space-y-3">
                    {airplanes.map((airplane) => (
                      <div 
                        key={airplane.id} 
                        className={`border border-aviation-blue/20 rounded-lg p-4 bg-secondary/20 hover:bg-secondary/30 transition-colors cursor-pointer ${selectedPlaneId === airplane.id ? 'ring-2 ring-aviation-blue' : ''}`}
                        onClick={() => setSelectedPlaneId(airplane.id)}
                      >
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <div className="font-semibold">{airplane.id}</div>
                            <Badge variant="outline" className={getStatusColor(airplane.status)}>
                              {airplane.status}
                            </Badge>
                          </div>
                          
                          <div className="text-sm text-muted-foreground">
                            {airplane.model}
                          </div>
                          
                          <div className="space-y-1">
                            <div className="flex justify-between text-sm">
                              <span className="text-muted-foreground">Fuel</span>
                              <span>{(planeInfoMap[airplane.id]?.fuelPct ?? airplane.fuel)}%</span>
                            </div>
                            <div className="w-full bg-secondary rounded-full h-1.5">
                              <div 
                                className="bg-aviation-amber h-1.5 rounded-full transition-all"
                                style={{ width: `${(planeInfoMap[airplane.id]?.fuelPct ?? airplane.fuel)}%` }}
                              />
                            </div>
                          </div>

                          <div className="space-y-1">
                            <div className="flex justify-between text-sm">
                              <span className="text-muted-foreground">Cargo</span>
                              <span>{(planeInfoMap[airplane.id]?.cargoPct ?? airplane.cargoLoad)}%</span>
                            </div>
                            <div className="w-full bg-secondary rounded-full h-1.5">
                              <div 
                                className="bg-aviation-blue h-1.5 rounded-full transition-all"
                                style={{ width: `${(planeInfoMap[airplane.id]?.cargoPct ?? airplane.cargoLoad)}%` }}
                              />
                            </div>
                          </div>
                          <div className="flex justify-end">
                            <Button variant="ghost" size="sm" onClick={(e) => { e.stopPropagation(); onAirplaneClick(airplane.id); }}>
                              Manage
                            </Button>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>

            {/* Dispatch Section */}
            {selectedPlaneId && (
              <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
                <CardHeader className="pb-3">
                  <CardTitle className="text-aviation-blue text-sm">Dispatch Plane {selectedPlaneId}</CardTitle>
                </CardHeader>
                <CardContent className="space-y-2 text-sm">
                  <div className="flex gap-2 items-center">
                    <Label className="mt-2">Destination</Label>
                    <select
                      className="flex-1 bg-secondary/50 border border-aviation-blue/20 rounded px-2 py-1"
                      value={dispatchDest}
                      onChange={(e) => setDispatchDest(e.target.value)}
                    >
                      <option value="">Select airport</option>
                      {airportsData.map(a => (
                        <option key={a.id} value={String(a.id)}>{a.id} - {a.name}</option>
                      ))}
                    </select>
                    {dispatchDest && (
                      <span className={`text-xs ${dispatchInfo?.ok ? 'text-green-400' : 'text-red-400'}`}>
                        {dispatchInfo ? (dispatchInfo.ok ? 'Reachable' : dispatchInfo.reason || 'Not reachable') : ''}
                      </span>
                    )}
                    <Button
                      variant="runway"
                      size="sm"
                      disabled={!dispatchDest}
                      onClick={async () => {
                        const { departPlane } = await import('@/api/game');
                        await departPlane(parseInt(selectedPlaneId, 10), parseInt(dispatchDest, 10));
                      }}
                    >
                      Depart
                    </Button>
                  </div>
                </CardContent>
              </Card>
            )}

            {/* Unload Controls */}
            {selectedPlaneId && (
              <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
                <CardHeader className="pb-3">
                  <CardTitle className="text-aviation-blue text-sm">Cargo Operations</CardTitle>
                </CardHeader>
                <CardContent>
                  <Button
                    variant="control"
                    size="sm"
                    onClick={async () => {
                      const { unloadAll } = await import('@/api/game');
                      await unloadAll(parseInt(selectedPlaneId, 10));
                    }}
                  >
                    Unload All Cargo
                  </Button>
                </CardContent>
              </Card>
            )}

            {/* Manifest List (per-order unload) */}
            {selectedPlaneId && manifest.length > 0 && (
              <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
                <CardHeader className="pb-3">
                  <CardTitle className="text-aviation-blue text-sm">Manifest</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2 text-sm">
                    {manifest.map(m => (
                      <div key={m.id} className="flex items-center justify-between border border-aviation-blue/20 rounded p-2 bg-secondary/20">
                        <div className="flex items-center gap-3">
                          <div className="font-semibold">{m.id}</div>
                          <div className="text-muted-foreground">to {m.destination}</div>
                          <div className="text-muted-foreground">{m.weight} kg</div>
                        </div>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={async () => {
                            const { unloadOrder, planeInfo, airportOrders } = await import('@/api/game');
                            await unloadOrder(parseInt(m.id, 10), parseInt(selectedPlaneId, 10));
                            const pinfo = await planeInfo(parseInt(selectedPlaneId, 10));
                            setManifest(pinfo.manifest.map(o => ({ id: String(o.id), cargoType: '', weight: o.weight, destination: String(o.destination_id), deadline: String(o.deadline), value: o.value })));
                            if (airportObj) {
                              const list = await airportOrders(airportObj.id);
                              setOrders(list.map(o => ({ id: String(o.id), cargoType: o.cargo_type || '', weight: o.weight, destination: String(o.destination_id), deadline: String(o.deadline), value: o.value })));
                            }
                          }}
                        >
                          Unload
                        </Button>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
            )}
          </div>

          {/* Right Panel - Orders */}
          <div className="col-span-8 space-y-4">
            
            {/* Filters */}
            <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
              <CardHeader className="pb-3">
                <CardTitle className="text-aviation-blue flex items-center gap-2">
                  <Filter className="w-5 h-5" />
                  Filter Orders
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-4 gap-4">
                  <div className="space-y-2">
                    <Label>Destination</Label>
                    <Input
                      placeholder="Filter by destination"
                      value={filterDestination}
                      onChange={(e) => setFilterDestination(e.target.value)}
                      className="bg-secondary/50 border-aviation-blue/20"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Max Weight (kg)</Label>
                    <Input
                      type="number"
                      placeholder="Max weight"
                      value={filterWeight}
                      onChange={(e) => setFilterWeight(e.target.value)}
                      className="bg-secondary/50 border-aviation-blue/20"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Min Value ($)</Label>
                    <Input
                      type="number"
                      placeholder="Min value"
                      value={filterValue}
                      onChange={(e) => setFilterValue(e.target.value)}
                      className="bg-secondary/50 border-aviation-blue/20"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Deadline After</Label>
                    <Input
                      type="date"
                      value={filterDeadline}
                      onChange={(e) => setFilterDeadline(e.target.value)}
                      className="bg-secondary/50 border-aviation-blue/20"
                    />
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Orders List */}
            <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
              <CardHeader className="pb-3">
                <CardTitle className="text-aviation-blue">
                  Available Orders ({filteredOrders.length})
                </CardTitle>
              </CardHeader>
              <CardContent>
                {selectedPlaneId && planeCapacity && (
                  <div className="mb-3 text-sm text-muted-foreground">
                    Selected plane {selectedPlaneId}: payload {planeCapacity.current.toFixed(0)} / {planeCapacity.capacity.toFixed(0)} kg
                  </div>
                )}
                {selectedPlaneId && (
                  <div className="mb-3">
                    <Button
                      variant="runway"
                      size="sm"
                      onClick={async () => {
                        const ids = Object.entries(selectedOrders).filter(([, v]) => v).map(([k]) => parseInt(k, 10));
                        for (const oid of ids) {
                          await apiLoad(oid, parseInt(selectedPlaneId, 10));
                        }
                        const idNum = parseInt(airportId, 10);
                        const list = await apiAirportOrders(idNum);
                        setOrders(list.map(o => ({ id: String(o.id), cargoType: "", weight: o.weight, destination: String(o.destination_id), deadline: String(o.deadline), value: o.value })));
                      }}
                    >
                      Load Selected Orders
                    </Button>
                    <Button
                      variant="control"
                      size="sm"
                      className="ml-2"
                      onClick={() => setSelectedOrders({})}
                    >
                      Clear Selection
                    </Button>
                  </div>
                )}
                <ScrollArea className="h-96">
                  <div className="space-y-3">
                    {filteredOrders.map((order) => (
                      <div 
                        key={order.id} 
                        className="border border-aviation-blue/20 rounded-lg p-4 bg-secondary/20 hover:bg-secondary/30 transition-colors"
                      >
                        <div className="flex items-center justify-between">
                          <div className="space-y-2">
                            <div className="flex items-center gap-4">
                              <input
                                type="checkbox"
                                className="mr-2"
                                checked={!!selectedOrders[order.id]}
                                onChange={(e) => setSelectedOrders(prev => ({ ...prev, [order.id]: e.target.checked }))}
                              />
                              <div className="font-semibold text-lg">{order.id}</div>
                              <Badge variant="outline" className="bg-aviation-blue/10 border-aviation-blue/30">
                                {order.cargoType}
                              </Badge>
                            </div>
                            
                            <div className="grid grid-cols-3 gap-4 text-sm">
                              <div className="flex items-center gap-1">
                                <Package className="w-3 h-3 text-muted-foreground" />
                                <span className="text-muted-foreground">Weight:</span>
                                <span className="font-medium">{order.weight} kg</span>
                              </div>
                              
                              <div className="flex items-center gap-1">
                                <MapPin className="w-3 h-3 text-muted-foreground" />
                                <span className="text-muted-foreground">To:</span>
                                <span className="font-medium">{order.destination}</span>
                              </div>
                              
                              <div className="flex items-center gap-1">
                                <Clock className="w-3 h-3 text-muted-foreground" />
                                <span className="text-muted-foreground">Due:</span>
                                <span className="font-medium">{order.deadline}</span>
                              </div>
                            </div>
                          </div>
                          
                          <div className="text-right">
                            <div className="text-aviation-radar font-bold text-xl">
                              ${order.value.toLocaleString()}
                            </div>
                            <div className="text-sm text-muted-foreground">
                              ${(order.value / order.weight).toFixed(2)}/kg
                            </div>
                            {selectedPlaneId && planeCapacity && (
                              <div className="mt-1 text-xs">
                                <Badge variant="outline" className={(planeCapacity.capacity - planeCapacity.current) >= order.weight ? 'bg-green-500/20 border-green-500/30 text-green-400' : 'bg-red-500/20 border-red-500/30 text-red-400'}>
                                  { (planeCapacity.capacity - planeCapacity.current) >= order.weight ? 'Fits payload' : 'Too heavy' }
                                </Badge>
                                <span className="mx-1" />
                                <Badge variant="outline" className={(canFlyCache[parseInt(order.destination, 10)] ?? false) ? 'bg-green-500/20 border-green-500/30 text-green-400' : 'bg-red-500/20 border-red-500/30 text-red-400'}>
                                  { (canFlyCache[parseInt(order.destination, 10)] ?? false) ? 'Can reach' : 'Route blocked' }
                                </Badge>
                              </div>
                            )}
                            <div className="mt-2">
                              <Button 
                                variant="runway" 
                                size="sm"
                                disabled={!selectedPlaneId}
                                onClick={async () => {
                                  if (!selectedPlaneId) return;
                                  await apiLoad(parseInt(order.id, 10), parseInt(selectedPlaneId, 10));
                                  const idNum = parseInt(airportId, 10);
                                  const list = await apiAirportOrders(idNum);
                                  setOrders(list.map(o => ({ id: String(o.id), cargoType: "", weight: o.weight, destination: String(o.destination_id), deadline: String(o.deadline), value: o.value })));
                                }}
                              >
                                Load to Plane {selectedPlaneId || '-'}
                              </Button>
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                    
                    {filteredOrders.length === 0 && (
                      <div className="text-center text-muted-foreground py-8">
                        No orders match your filter criteria
                      </div>
                    )}
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
};
