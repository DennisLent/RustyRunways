import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { 
  ArrowLeft, 
  Plane, 
  Fuel, 
  Package, 
  Send, 
  Wrench,
  Filter,
  Plus,
  Minus,
  MapPin,
  CircleDollarSign
} from "lucide-react";
import { airportOrders as apiAirportOrders, planeInfo as apiPlaneInfo, departPlane as apiDepart, loadOrder as apiLoad, unloadOrder as apiUnload, refuelPlane as apiRefuel, maintenance as apiMaint, canFly as apiCanFly, reachability as apiReach, sellPlane as apiSell } from "@/api/game";

type PayloadKind = 'cargo' | 'passengers';

interface Order {
  id: string;
  payloadKind: PayloadKind;
  cargoType?: string;
  weight?: number;
  passengerCount?: number;
  destination: string;
  deadline: string;
  value: number;
}

interface Airport {
  id: string;
  name: string;
  code: string;
  canReach: boolean;
  distance: number;
  fuelRequired: number;
}

interface AirplaneDetailScreenProps {
  airplaneId: string;
  onBack: () => void;
  airportsData?: { id: number; name: string }[];
  onSold?: (planeId: string, refund: number) => void | Promise<void>;
}

export const AirplaneDetailScreen = ({ 
  airplaneId, 
  onBack,
  airportsData,
  onSold
}: AirplaneDetailScreenProps) => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedDestination, setSelectedDestination] = useState("");
  const [filterDestination, setFilterDestination] = useState("");
  const [filterWeight, setFilterWeight] = useState("");
  const [filterValue, setFilterValue] = useState("");
  const [canFlyCache, setCanFlyCache] = useState<Record<number, boolean>>({});
  const [reachInfo, setReachInfo] = useState<{ ok: boolean; reason?: string } | null>(null);
  // Reachability for all airports (for destination selector)
  const [airportReachCache, setAirportReachCache] = useState<Record<number, boolean>>({});

  // Live data
  const [airplane, setAirplane] = useState({
    id: airplaneId,
    model: "",
    location: "",
    status: "parked",
    fuel: 0,
    maxFuel: 100,
    cargoCapacity: 0,
    currentCargo: 0,
    passengerCapacity: 0,
    currentPassengers: 0,
    condition: 100,
    loadedOrders: [] as Order[],
  });

  const [availableOrders, setAvailableOrders] = useState<Order[]>([]);

  // Keep filteredOrders defined before effects that reference it
  const filteredOrders = availableOrders.filter(order => {
    const maxW = filterWeight ? parseInt(filterWeight, 10) : Infinity;
    const minV = filterValue ? parseInt(filterValue, 10) : 0;
    const weightOk = order.payloadKind === 'cargo'
      ? (order.weight ?? 0) <= maxW
      : true;
    return (
      (!filterDestination || order.destination.toLowerCase().includes(filterDestination.toLowerCase())) &&
      weightOk &&
      order.value >= minV
    );
  });

  const payloadSummary = (order: Order) => {
    if (order.payloadKind === 'cargo') {
      const weight = order.weight ?? 0;
      return `${order.cargoType ?? 'Cargo'} • ${weight.toLocaleString()} kg`;
    }
    return `Passengers • ${(order.passengerCount ?? 0).toLocaleString()} pax`;
  };

  const orderCapacityOk = (order: Order) => {
    if (order.payloadKind === 'cargo') {
      return airplane.currentCargo + (order.weight ?? 0) <= airplane.cargoCapacity;
    }
    return airplane.currentPassengers + (order.passengerCount ?? 0) <= airplane.passengerCapacity;
  };

  const capacityBadgeLabel = (order: Order) => {
    if (order.payloadKind === 'cargo') {
      return orderCapacityOk(order) ? 'Fits cargo' : 'Too heavy';
    }
    return orderCapacityOk(order) ? 'Seats available' : 'No seats';
  };

  const availableAirports: Airport[] = airportsData
    ? airportsData.map(a => ({
        id: String(a.id),
        name: a.name,
        code: String(a.id),
        canReach: airportReachCache[a.id] ?? true,
        distance: 0,
        fuelRequired: 0,
      }))
    : [
      { id: "1", name: "Los Angeles Intl", code: "1", canReach: true, distance: 0, fuelRequired: 0 },
    ];

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const idNum = parseInt(airplaneId, 10);
      if (!Number.isFinite(idNum)) {
        throw new Error("Invalid airplane id");
      }
      const info = await apiPlaneInfo(idNum);
      const fuelPct = info.fuel_capacity > 0 ? Math.round((info.fuel_current / info.fuel_capacity) * 100) : 0;
      setAirplane(prev => ({
        ...prev,
        id: String(info.id),
        model: info.model,
        location: info.current_airport_id != null ? String(info.current_airport_id) : "",
        status: info.status,
        fuel: fuelPct,
        maxFuel: 100,
        cargoCapacity: info.payload_capacity,
        currentCargo: info.payload_current,
        passengerCapacity: info.passenger_capacity,
        currentPassengers: info.passenger_current,
        loadedOrders: info.manifest.map(o => ({
          id: String(o.id),
          payloadKind: (o.payload_kind || 'cargo') as PayloadKind,
          cargoType: o.cargo_type || undefined,
          weight: o.weight ?? undefined,
          passengerCount: o.passenger_count ?? undefined,
          destination: String(o.destination_id),
          deadline: String(o.deadline),
          value: o.value,
        }))
      }));

      if (info.current_airport_id != null) {
        const orders = await apiAirportOrders(info.current_airport_id);
        setAvailableOrders(orders.map(o => ({
          id: String(o.id),
          payloadKind: (o.payload_kind || 'cargo') as PayloadKind,
          cargoType: o.cargo_type || undefined,
          weight: o.weight ?? undefined,
          passengerCount: o.passenger_count ?? undefined,
          destination: String(o.destination_id),
          deadline: String(o.deadline),
          value: o.value,
        })));
      } else {
        setAvailableOrders([]);
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [airplaneId]);

  // Compute canFly for all airports to gray out unreachable destinations
  useEffect(() => {
    (async () => {
      if (!airportsData || airportsData.length === 0) return;
      const pid = parseInt(airplane.id, 10);
      if (!Number.isFinite(pid)) return;
      const next: Record<number, boolean> = {};
      for (const a of airportsData) {
        try {
          next[a.id] = await apiCanFly(pid, a.id);
        } catch (_) { void 0 }
      }
      setAirportReachCache(next);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [airplane.id, airportsData && airportsData.length]);

  useEffect(() => {
    // compute reachability per destination for order list
    (async () => {
      const dests = Array.from(new Set(filteredOrders.map(o => parseInt(o.destination, 10))));
      const cache: Record<number, boolean> = {};
      for (const d of dests) {
        try {
          cache[d] = await apiCanFly(parseInt(airplane.id, 10), d);
        } catch (_) { void 0 }
      }
      setCanFlyCache(cache);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [airplane.id, filteredOrders.length]);

  useEffect(() => {
    (async () => {
      if (!selectedDestination) { setReachInfo(null); return; }
      try {
        const info = await apiReach(parseInt(airplane.id, 10), parseInt(selectedDestination, 10));
        setReachInfo(info);
      } catch (_) { setReachInfo(null); }
    })();
  }, [selectedDestination, airplane.id]);

  // filteredOrders defined earlier

  async function handleMaintenance() {
    await apiMaint(parseInt(airplane.id, 10));
    await refresh();
  }

  async function handleRefuel() {
    await apiRefuel(parseInt(airplane.id, 10));
    await refresh();
  }

  async function handleSell() {
    try {
      setError(null);
      const refund = await apiSell(parseInt(airplane.id, 10));
      if (onSold) {
        await onSold(airplane.id, refund);
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function handleDispatch() {
    if (!selectedDestination) return;
    await apiDepart(parseInt(airplane.id, 10), parseInt(selectedDestination, 10));
    await refresh();
  }

  async function handleUnload(orderId: string) {
    await apiUnload(parseInt(orderId, 10), parseInt(airplane.id, 10));
    await refresh();
  }

  async function handleLoad(orderId: string) {
    await apiLoad(parseInt(orderId, 10), parseInt(airplane.id, 10));
    await refresh();
  }

  return (
    <div className="min-h-screen bg-gradient-control p-4">
      <div className="max-w-7xl mx-auto space-y-4">
        {loading && (
          <div className="text-muted-foreground">Loading aircraft details...</div>
        )}
        {error && (
          <div className="text-red-400">{error}</div>
        )}
        
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Button variant="control" onClick={onBack}>
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back
            </Button>
            <div className="flex items-center gap-2">
              <div className="p-2 rounded-lg bg-aviation-blue/20 border border-aviation-blue/30">
                <Plane className="w-5 h-5 text-aviation-blue" />
              </div>
              <div>
                <h1 className="text-2xl font-bold text-foreground">{airplane.model}</h1>
                <p className="text-muted-foreground">ID: {airplane.id} • Location: {airplane.location}</p>
              </div>
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <Button variant="control" onClick={handleMaintenance}>
              <Wrench className="w-4 h-4 mr-1" />
              Maintenance
            </Button>
            <Button variant="runway" onClick={handleRefuel}>
              <Fuel className="w-4 h-4 mr-1" />
              Refuel
            </Button>
            <Button
              variant="destructive"
              onClick={handleSell}
              disabled={airplane.loadedOrders.length > 0 || airplane.status.toLowerCase() !== 'parked'}
            >
              <CircleDollarSign className="w-4 h-4 mr-1" />
              Sell
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-12 gap-4">
          
          {/* Left Panel - Airplane Status */}
          <div className="col-span-4 space-y-4">
            
            {/* Status Card */}
            <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
              <CardHeader className="pb-3">
                <CardTitle className="text-aviation-blue">Aircraft Status</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Status</span>
                    <Badge variant="outline" className="bg-green-500/20 text-green-400 border-green-500/30">
                      {airplane.status}
                    </Badge>
                  </div>
                  
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Fuel</span>
                      <span>{airplane.fuel}%</span>
                    </div>
                    <div className="w-full bg-secondary rounded-full h-2">
                      <div 
                        className="bg-aviation-amber h-2 rounded-full transition-all"
                        style={{ width: `${airplane.fuel}%` }}
                      />
                    </div>
                  </div>

                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Cargo Load</span>
                      <span>{airplane.cargoCapacity > 0 ? Math.round((airplane.currentCargo / airplane.cargoCapacity) * 100) : 0}%</span>
                    </div>
                    <div className="w-full bg-secondary rounded-full h-2">
                      <div 
                        className="bg-aviation-blue h-2 rounded-full transition-all"
                        style={{ width: `${airplane.cargoCapacity > 0 ? (airplane.currentCargo / airplane.cargoCapacity) * 100 : 0}%` }}
                      />
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {airplane.currentCargo.toLocaleString()} / {airplane.cargoCapacity.toLocaleString()} kg
                    </div>
                  </div>

                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Passengers</span>
                      <span>{airplane.passengerCapacity > 0 ? Math.round((airplane.currentPassengers / airplane.passengerCapacity) * 100) : 0}%</span>
                    </div>
                    <div className="w-full bg-secondary rounded-full h-2">
                      <div
                        className="bg-aviation-radar h-2 rounded-full transition-all"
                        style={{ width: `${airplane.passengerCapacity > 0 ? (airplane.currentPassengers / airplane.passengerCapacity) * 100 : 0}%` }}
                      />
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {airplane.currentPassengers.toLocaleString()} / {airplane.passengerCapacity.toLocaleString()} pax
                    </div>
                  </div>

                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Condition</span>
                      <span>{airplane.condition}%</span>
                    </div>
                    <div className="w-full bg-secondary rounded-full h-2">
                      <div 
                        className="bg-aviation-radar h-2 rounded-full transition-all"
                        style={{ width: `${airplane.condition}%` }}
                      />
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Dispatch Card */}
            <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
              <CardHeader className="pb-3">
                <CardTitle className="text-aviation-blue">Dispatch Aircraft</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label>Destination Airport</Label>
                  <Select onValueChange={setSelectedDestination}>
                    <SelectTrigger className="bg-secondary/50 border-aviation-blue/20">
                      <SelectValue placeholder="Select destination" />
                    </SelectTrigger>
                    <SelectContent>
                      {availableAirports.map((airport) => (
                        <SelectItem 
                          key={airport.id} 
                          value={airport.id}
                          disabled={!airport.canReach}
                        >
                          <div className="flex items-center justify-between w-full">
                            <span>{airport.code} - {airport.name}</span>
                            {!airport.canReach && (
                              <Badge variant="destructive" className="ml-2">
                                Out of Range
                              </Badge>
                            )}
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                {selectedDestination && (
                  <div className="text-sm space-y-1 p-3 bg-secondary/20 rounded-lg">
                    <div className={reachInfo?.ok ? 'text-green-400' : 'text-red-400'}>
                      {reachInfo ? (reachInfo.ok ? 'Reachable' : reachInfo.reason || 'Not reachable') : ''}
                    </div>
                  </div>
                )}

                <Button 
                  className="w-full" 
                  variant="runway"
                  disabled={!selectedDestination}
                  onClick={handleDispatch}
                >
                  <Send className="w-4 h-4 mr-2" />
                  Dispatch to {selectedDestination}
                </Button>
              </CardContent>
            </Card>
          </div>

          {/* Right Panel - Cargo Management */}
          <div className="col-span-8">
            <Tabs defaultValue="loaded" className="space-y-4">
              <TabsList className="grid w-full grid-cols-2 bg-secondary/50">
                <TabsTrigger value="loaded">Loaded Payload ({airplane.loadedOrders.length})</TabsTrigger>
                <TabsTrigger value="available">Available Payload ({filteredOrders.length})</TabsTrigger>
              </TabsList>

              <TabsContent value="loaded" className="space-y-4">
                <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
                  <CardHeader className="pb-3">
                    <CardTitle className="text-aviation-blue">Loaded Orders</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <ScrollArea className="h-96">
                      <div className="space-y-3">
                        {airplane.loadedOrders.map((order) => (
                          <div 
                            key={order.id} 
                            className="border border-aviation-blue/20 rounded-lg p-4 bg-secondary/20"
                          >
                            <div className="flex items-center justify-between">
                              <div className="space-y-1">
                                <div className="font-semibold">{order.id}</div>
                                <div className="text-sm text-muted-foreground">
                                  {payloadSummary(order)}
                                </div>
                                <div className="flex items-center gap-4 text-sm">
                                  <span className="flex items-center gap-1">
                                    <MapPin className="w-3 h-3" />
                                    {order.destination}
                                  </span>
                                  <span>Due: {order.deadline}</span>
                                </div>
                              </div>
                              <div className="text-right space-y-2">
                                <div className="text-aviation-radar font-semibold">
                                  ${order.value.toLocaleString()}
                                </div>
                                <Button 
                                  variant="warning" 
                                  size="sm"
                                  onClick={() => handleUnload(order.id)}
                                >
                                  <Minus className="w-3 h-3 mr-1" />
                                  Unload
                                </Button>
                              </div>
                            </div>
                          </div>
                        ))}
                      </div>
                    </ScrollArea>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="available" className="space-y-4">
                
                {/* Filters */}
                <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
                  <CardHeader className="pb-3">
                    <CardTitle className="text-aviation-blue flex items-center gap-2">
                      <Filter className="w-5 h-5" />
                      Filter Orders
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-3 gap-4">
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
                        <Label>Max Cargo Weight (kg)</Label>
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
                    </div>
                  </CardContent>
                </Card>

                {/* Available Orders */}
                <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
                  <CardHeader className="pb-3">
                    <CardTitle className="text-aviation-blue">Available Orders</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <ScrollArea className="h-96">
                      <div className="space-y-3">
                        {filteredOrders.map((order) => (
                          <div 
                            key={order.id} 
                            className="border border-aviation-blue/20 rounded-lg p-4 bg-secondary/20 hover:bg-secondary/30 transition-colors"
                          >
                            <div className="flex items-center justify-between">
                              <div className="space-y-1">
                                <div className="font-semibold">{order.id}</div>
                                <div className="text-sm text-muted-foreground">
                                  {payloadSummary(order)}
                                </div>
                                <div className="flex items-center gap-4 text-sm">
                                  <span className="flex items-center gap-1">
                                    <MapPin className="w-3 h-3" />
                                    {order.destination}
                                  </span>
                                  <span>Due: {order.deadline}</span>
                                </div>
                              </div>
                              <div className="text-right space-y-2">
                                <div className="text-aviation-radar font-semibold">
                                  ${order.value.toLocaleString()}
                                </div>
                                <div className="mb-2">
                                  <Badge variant="outline" className={orderCapacityOk(order) ? 'bg-green-500/20 border-green-500/30 text-green-400' : 'bg-red-500/20 border-red-500/30 text-red-400'}>
                                    {capacityBadgeLabel(order)}
                                  </Badge>
                                  <span className="mx-1" />
                                  <Badge variant="outline" className={(canFlyCache[parseInt(order.destination, 10)] ?? false) ? 'bg-green-500/20 border-green-500/30 text-green-400' : 'bg-red-500/20 border-red-500/30 text-red-400'}>
                                    {(canFlyCache[parseInt(order.destination, 10)] ?? false) ? 'Can reach' : 'Route blocked'}
                                  </Badge>
                                </div>
                                <Button 
                                  variant="control" 
                                  size="sm"
                                  onClick={() => handleLoad(order.id)}
                                  disabled={!orderCapacityOk(order)}
                                >
                                  <Plus className="w-3 h-3 mr-1" />
                                  Load
                                </Button>
                              </div>
                            </div>
                          </div>
                        ))}
                      </div>
                    </ScrollArea>
                  </CardContent>
                </Card>
              </TabsContent>
            </Tabs>
          </div>
        </div>
      </div>
    </div>
  );
};
