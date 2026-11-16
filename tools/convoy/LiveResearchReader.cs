// Experimental observation for one fingerprinted build. No writes, hooks, injection or remote threads.
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.IO.MemoryMappedFiles;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using System.Web.Script.Serialization;

class LiveResearchReader {
    const uint ReadRights=0x410; // PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, never VM_WRITE/VM_OPERATION.
    const string ExpectedHash="055ec5343d53bd427de54b06eca74937f0d348dbb0c5d71862edddcf45e2db66";
    const long TrafficSlotRva=0x3715000, SignatureRva=0x53336c;
    static readonly int[] Pattern={0x48,0x8b,0xd9,0x48,0x8b,0x0d,-1,-1,-1,-1,0x48,0x85,0xc9,0x74,-1,0x48,0x8b,0x83,-1,-1,-1,-1,0x48,0x85,0xc0};
    [StructLayout(LayoutKind.Sequential)] struct MBI {public IntPtr BaseAddress,AllocationBase;public uint AllocationProtect;public UIntPtr RegionSize;public uint State,Protect,Type;}
    [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr OpenProcess(uint access,bool inherit,int pid);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr handle,out uint code);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool ReadProcessMemory(IntPtr process,IntPtr address,[Out]byte[] bytes,UIntPtr size,out UIntPtr read);
    [DllImport("kernel32.dll",SetLastError=true)] static extern UIntPtr VirtualQueryEx(IntPtr process,IntPtr address,out MBI info,UIntPtr length);
    [DllImport("kernel32.dll",EntryPoint="QueryFullProcessImageNameW",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool QueryFullProcessImageName(IntPtr process,uint flags,StringBuilder path,ref uint size);
    [DllImport("psapi.dll",SetLastError=true)] static extern bool EnumProcessModulesEx(IntPtr process,[Out]IntPtr[] modules,uint bytes,out uint needed,uint filter);
    static readonly JavaScriptSerializer Json=new JavaScriptSerializer();
    static string Salt=Guid.NewGuid().ToString("N");
    static void Emit(object value){Console.WriteLine(Json.Serialize(value));}
    static string Digest(byte[] bytes){using(var sha=SHA256.Create())return BitConverter.ToString(sha.ComputeHash(bytes)).Replace("-","").ToLowerInvariant();}
    static string SafeReason(Exception error){string text=error.Message;return System.Text.RegularExpressions.Regex.IsMatch(text,"^[a-z][a-z0-9-]{0,63}$")?text:"internal-error";}
    static bool Valid(long at){return at>=0x10000&&at<=0x7fffffffffff;}
    sealed class Memory {
        readonly IntPtr process;public Memory(IntPtr p){process=p;}
        public byte[] Read(long at,int size){
            if(!Valid(at)||size<1||size>4096||at>0x7fffffffffff-size)throw new Exception("invalid-range");
            MBI info;
            if(VirtualQueryEx(process,new IntPtr(at),out info,new UIntPtr((uint)Marshal.SizeOf(typeof(MBI))))==UIntPtr.Zero)throw new Exception("query-failed");
            long start=info.BaseAddress.ToInt64(),length=checked((long)info.RegionSize.ToUInt64());
            if(info.State!=0x1000||(info.Protect&0x101)!=0||(info.Protect&0xee)==0||at<start||at-start>length-size)throw new Exception("unreadable-range");
            byte[] bytes=new byte[size];UIntPtr read;
            if(!ReadProcessMemory(process,new IntPtr(at),bytes,new UIntPtr((uint)size),out read)||read.ToUInt64()!=(ulong)size)throw new Exception("read-failed");
            return bytes;
        }
        public long Ptr(long at){long p=BitConverter.ToInt64(Read(at,8),0);if(!Valid(p)||(p&7)!=0)throw new Exception("invalid-pointer");return p;}
        public ulong U64(long at){return BitConverter.ToUInt64(Read(at,8),0);}
        public string TypeName(long obj,long image){
            try {
                long vtable=Ptr(obj),col=Ptr(vtable-8);byte[] c=Read(col,24);
                if(BitConverter.ToUInt32(c,0)!=1||col-BitConverter.ToUInt32(c,20)!=image)return "unknown";
                long type=image+BitConverter.ToUInt32(c,12)+16;var name=new StringBuilder();
                for(int i=0;i<120;i++){byte ch=Read(type+i,1)[0];if(ch==0)break;if(ch<32||ch>126)return "unknown";name.Append((char)ch);}
                string text=name.ToString();return text.StartsWith(".?AV",StringComparison.Ordinal)||text.StartsWith(".?AU",StringComparison.Ordinal)?text:"unknown";
            }catch{return "unknown";}
        }
    }
    static Dictionary<string,object> Placement(byte[] bytes){
        double x=BitConverter.ToSingle(bytes,0)+BitConverter.ToInt16(bytes,12)*512.0;
        double y=BitConverter.ToSingle(bytes,4),z=BitConverter.ToSingle(bytes,8)+BitConverter.ToInt16(bytes,14)*512.0;
        var q=new double[4];double norm=0;for(int i=0;i<4;i++){q[i]=BitConverter.ToSingle(bytes,16+i*4);norm+=q[i]*q[i];}
        if(Double.IsNaN(x)||Double.IsNaN(y)||Double.IsNaN(z)||Double.IsInfinity(x)||Double.IsInfinity(y)||Double.IsInfinity(z)||Double.IsNaN(norm)||Double.IsInfinity(norm)||Math.Abs(norm-1)>0.02||Math.Abs(x)>17000000||Math.Abs(z)>17000000||Math.Abs(y)>100000)throw new Exception("invalid-placement");
        return new Dictionary<string,object>{{"x",x},{"y",y},{"z",z},{"quaternion",q}};
    }
    sealed class PublicationFreshness {
        int lastPid;uint lastSequence;long progressedAt;bool initialized,progressed;
        public void Reset(){initialized=false;progressed=false;}
        public bool Accept(int pid,uint sequence,long now){
            if(!initialized||pid!=lastPid||sequence<lastSequence){lastPid=pid;lastSequence=sequence;progressedAt=now;initialized=true;progressed=false;return false;}
            if(sequence!=lastSequence){lastSequence=sequence;progressedAt=now;progressed=true;}
            return progressed&&now>=progressedAt&&now-progressedAt<=1500;
        }
    }
    static readonly PublicationFreshness OwnFreshness=new PublicationFreshness();
    static readonly Stopwatch OwnClock=Stopwatch.StartNew();
    static object OwnPose(int pid){
        try {
            using(var map=MemoryMappedFile.OpenExisting("Local\\WestPlay.Bridge.v29",MemoryMappedFileRights.Read))
            using(var view=map.CreateViewAccessor(0,128,MemoryMappedFileAccess.Read)){
                if(view.ReadUInt32(0)!=0x37425057||view.ReadUInt32(4)!=29||view.ReadUInt32(8)!=(uint)pid){OwnFreshness.Reset();return null;}
                uint before=view.ReadUInt32(72);if((before&1)!=0)return null;
                Thread.MemoryBarrier();byte[] data=new byte[48];view.ReadArray(80,data,0,data.Length);Thread.MemoryBarrier();
                if(before!=view.ReadUInt32(72))return null;
                if(!OwnFreshness.Accept(pid,before,OwnClock.ElapsedMilliseconds))return null;
                double x=BitConverter.ToDouble(data,0),y=BitConverter.ToDouble(data,8),z=BitConverter.ToDouble(data,16);
                if(Double.IsNaN(x)||Double.IsNaN(y)||Double.IsNaN(z)||Double.IsInfinity(x)||Double.IsInfinity(y)||Double.IsInfinity(z)||Math.Abs(x)>17000000||Math.Abs(z)>17000000||Math.Abs(y)>100000)return null;
                return new {x=x,y=y,z=z,heading=BitConverter.ToSingle(data,24),speed=BitConverter.ToSingle(data,36),frameCounter=view.ReadUInt64(16)};
            }
        }catch{OwnFreshness.Reset();return null;}
    }
    sealed class CandidateHeader {public int Offset;public long Data;public ulong Count,Capacity;}
    static List<CandidateHeader> FindHeaders(byte[] prefix){
        if(prefix==null||prefix.Length!=0x800)throw new Exception("header-fixture-size");
        var headers=new List<CandidateHeader>();
        for(int offset=0x40;offset<=0x7d8;offset+=8){
            long data=BitConverter.ToInt64(prefix,offset);ulong count=BitConverter.ToUInt64(prefix,offset+8),capacity=BitConverter.ToUInt64(prefix,offset+16);
            if(Valid(data)&&(data&7)==0&&count>0&&count<=128&&capacity>=count&&capacity<=4096){
                if(headers.Count>=64)throw new Exception("header-budget");
                headers.Add(new CandidateHeader{Offset=offset,Data=data,Count=count,Capacity=capacity});
            }
        }
        return headers;
    }
    const long SessionSlotRva=0x3715040;
    static long NamePointer(Func<long,int,byte[]> read,long at) {
        long value=BitConverter.ToInt64(read(at,8),0);
        if(!Valid(value)||(value&7)!=0)throw new Exception("name-invalid-pointer");return value;
    }
    static string DisplayName(Func<long,int,byte[]> read,long pointer) {
        // Names are UTF-8 in this fingerprint. Do not infer SCS string header length/capacity.
        var bytes=new List<byte>();bool terminated=false;
        for(int i=0;i<=256;i++){byte b=read(pointer+i,1)[0];if(b==0){terminated=true;break;}if(i==256)break;bytes.Add(b);}
        if(!terminated)throw new Exception("name-too-long");
        string decoded=new UTF8Encoding(false,true).GetString(bytes.ToArray());var clean=new StringBuilder();
        foreach(char c in decoded){UnicodeCategory category=CharUnicodeInfo.GetUnicodeCategory(c);if(category!=UnicodeCategory.Control&&category!=UnicodeCategory.Format&&category!=UnicodeCategory.LineSeparator&&category!=UnicodeCategory.ParagraphSeparator)clean.Append(c);}
        string value=clean.ToString().Trim();
        return value.Length==0||value.StartsWith("@@",StringComparison.Ordinal)?null:value;
    }
    sealed class NameLink {public long Info,Remote,Actor,Text;public string Name;}
    static Dictionary<long,string> ResolveNames(Func<long,int,byte[]> read,long image,HashSet<long> actors) {
        var names=new Dictionary<long,string>();
        long session=BitConverter.ToInt64(read(image+SessionSlotRva,8),0);if(session==0)return names;
        if(!Valid(session)||(session&7)!=0)throw new Exception("name-invalid-session");
        ulong count=BitConverter.ToUInt64(read(session+0x2e8,8),0);if(count>128)throw new Exception("name-roster-bounds");
        long data=count>0?NamePointer(read,session+0x2e0):BitConverter.ToInt64(read(session+0x2e0,8),0);
        byte[] pointers=count>0?read(data,(int)count*8):new byte[0];
        var seenInfo=new HashSet<long>();var seenActor=new HashSet<long>();var links=new List<NameLink>();
        for(int i=0;i<(int)count;i++){
            long info=BitConverter.ToInt64(pointers,i*8);if(!Valid(info)||(info&7)!=0||!seenInfo.Add(info))throw new Exception("name-roster-duplicate");
            long remote=BitConverter.ToInt64(read(info+0x430,8),0);if(remote==0)continue;if(!Valid(remote)||(remote&7)!=0)throw new Exception("name-invalid-remote");
            long actor=BitConverter.ToInt64(read(remote+0x290,8),0);if(actor==0)continue;if(!Valid(actor)||(actor&7)!=0||!seenActor.Add(actor))throw new Exception("name-actor-ambiguous");
            // The game's native map path requires these actor flags. No fallback coordinates/names.
            if(!actors.Contains(actor)||(read(actor+0x228,1)[0]&0x60)!=0x60)continue;
            long text=BitConverter.ToInt64(read(info+0x110,8),0);if(!Valid(text))throw new Exception("name-invalid-text");
            string name=DisplayName(read,text);
            links.Add(new NameLink{Info=info,Remote=remote,Actor=actor,Text=text,Name=name});
        }
        // Validate the whole snapshot after all strings have been decoded; never retain a cached name.
        if(BitConverter.ToInt64(read(image+SessionSlotRva,8),0)!=session||BitConverter.ToUInt64(read(session+0x2e8,8),0)!=count||BitConverter.ToInt64(read(session+0x2e0,8),0)!=data)throw new Exception("name-snapshot-changed");
        byte[] after=count>0?read(data,(int)count*8):new byte[0];for(int i=0;i<pointers.Length;i++)if(pointers[i]!=after[i])throw new Exception("name-snapshot-changed");
        foreach(var link in links){
            if(NamePointer(read,link.Info+0x430)!=link.Remote||NamePointer(read,link.Remote+0x290)!=link.Actor||BitConverter.ToInt64(read(link.Info+0x110,8),0)!=link.Text||(read(link.Actor+0x228,1)[0]&0x60)!=0x60||DisplayName(read,link.Text)!=link.Name)throw new Exception("name-chain-changed");
            if(link.Name!=null)names.Add(link.Actor,link.Name);
        }
        return names;
    }
    static bool NameSignatures(Memory memory,long image) {
        long[] offsets={0xf5568c,0xf556c3,0xf556dc,0xf565d0,0x537127};
        string[] expected={"488bba30040000","488b8f90020000","0f104128","498d8fd8020000483b59100f830d020000488b4108488b14d8","488d9008010000488bcbe89a0cbcff"};
        try{for(int i=0;i<offsets.Length;i++){byte[] actual=memory.Read(image+offsets[i],expected[i].Length/2);if(BitConverter.ToString(actual).Replace("-","").ToLowerInvariant()!=expected[i])return false;}return true;}catch{return false;}
    }
    static void NameSelfTest() {
        // Synthetic address space: no game is opened or called by this test.
        var blocks=new Dictionary<long,byte[]>();
        long image=0x100000,session=0x5000000,data=0x5001000,info=0x5002000,remote=0x5003000,actor=0x5004000,text=0x5005000;
        blocks[image+0x3715040]=BitConverter.GetBytes(session);blocks[session+0x2e0]=BitConverter.GetBytes(data);blocks[session+0x2e8]=BitConverter.GetBytes(1UL);
        blocks[data]=BitConverter.GetBytes(info);blocks[info+0x430]=BitConverter.GetBytes(remote);blocks[remote+0x290]=BitConverter.GetBytes(actor);blocks[actor+0x228]=new byte[]{0x60};blocks[info+0x110]=BitConverter.GetBytes(text);
        blocks[text]=Encoding.UTF8.GetBytes("Çağrı 🚚\0");
        Func<long,int,byte[]> read=delegate(long at,int size){foreach(var pair in blocks){if(at>=pair.Key&&at-pair.Key<=pair.Value.Length-size){var b=new byte[size];Array.Copy(pair.Value,(int)(at-pair.Key),b,0,size);return b;}}throw new Exception("fixture-range");};
        var actors=new HashSet<long>{actor};
        if(ResolveNames(read,image,actors)[actor]!="Çağrı 🚚")throw new Exception("name-exact-actor");
        if(ResolveNames(read,image,new HashSet<long>{actor+8}).Count!=0)throw new Exception("name-no-nearest-join");
        blocks[text]=new byte[]{0xc0,0xaf,0};bool rejected=false;try{ResolveNames(read,image,actors);}catch{rejected=true;}if(!rejected)throw new Exception("name-invalid-utf8");
        blocks[text]=new byte[257];for(int i=0;i<257;i++)blocks[text][i]=65;rejected=false;try{ResolveNames(read,image,actors);}catch{rejected=true;}if(!rejected)throw new Exception("name-byte-budget");
        blocks[text]=Encoding.UTF8.GetBytes("A\u202eB\n\0");if(ResolveNames(read,image,actors)[actor]!="AB")throw new Exception("name-control-filter");
        blocks[session+0x2e8]=BitConverter.GetBytes(2UL);blocks[data]=new byte[16];Array.Copy(BitConverter.GetBytes(info),0,blocks[data],0,8);Array.Copy(BitConverter.GetBytes(info+0x800),0,blocks[data],8,8);
        blocks[info+0x800+0x430]=BitConverter.GetBytes(remote);rejected=false;try{ResolveNames(read,image,actors);}catch{rejected=true;}if(!rejected)throw new Exception("name-ambiguous-actor");
        blocks[session+0x2e8]=BitConverter.GetBytes(1UL);int calls=0;Func<long,int,byte[]> changed=delegate(long at,int size){if(at==info+0x110&&++calls>1)return BitConverter.GetBytes(text+8);return read(at,size);};
        rejected=false;try{ResolveNames(changed,image,actors);}catch{rejected=true;}if(!rejected)throw new Exception("name-changing-chain");
        blocks[actor+0x228]=new byte[]{0};if(ResolveNames(read,image,actors).Count!=0)throw new Exception("name-inactive-actor");blocks[actor+0x228]=new byte[]{0x60};
        blocks[session+0x2e8]=BitConverter.GetBytes(0UL);if(ResolveNames(read,image,actors).Count!=0)throw new Exception("name-empty-roster");
        blocks[session+0x2e8]=BitConverter.GetBytes(1UL);calls=0;Func<long,int,byte[]> changingHeader=delegate(long at,int size){if(at==session+0x2e8&&++calls>1)return BitConverter.GetBytes(2UL);return read(at,size);};
        rejected=false;try{ResolveNames(changingHeader,image,actors);}catch{rejected=true;}if(!rejected)throw new Exception("name-changing-roster");
        blocks[session+0x2e8]=BitConverter.GetBytes(129UL);rejected=false;try{ResolveNames(read,image,actors);}catch{rejected=true;}if(!rejected)throw new Exception("name-roster-budget");
    }
    static int SelfTest(){
        NameSelfTest();
        using(var ownLifetime=new ParentLifetime(Process.GetCurrentProcess().Id)){if(!ownLifetime.Alive())throw new Exception("parent-own-process-fixture");}
        if(!ParentAlive(delegate{return false;},delegate{return 10L;},10)||ParentAlive(delegate{return true;},delegate{return 10L;},10)||ParentAlive(delegate{return false;},delegate{return 11L;},10)||ParentAlive(delegate{throw new Exception("fixture");},delegate{return 10L;},10))throw new Exception("parent-lifetime-fixture");
        if(SafeReason(new System.IO.IOException(@"C:\Users\private\game.exe"))!="internal-error")throw new Exception("error-redaction");
        var freshness=new PublicationFreshness();
        if(freshness.Accept(1,2,0)||!freshness.Accept(1,4,500)||freshness.Accept(1,4,2101)||!freshness.Accept(1,6,2200)||freshness.Accept(2,8,2300)||freshness.Accept(2,2,2400)||!freshness.Accept(2,4,2500))throw new Exception("publication-freshness");
        if(Marshal.SizeOf(typeof(MBI))!=48||ReadRights!=0x410)throw new Exception("rights-or-structure");
        byte[] headerFixture=new byte[0x800];Array.Copy(BitConverter.GetBytes(0x20001000L),0,headerFixture,0x218,8);Array.Copy(BitConverter.GetBytes(2UL),0,headerFixture,0x220,8);Array.Copy(BitConverter.GetBytes(2UL),0,headerFixture,0x228,8);
        if(FindHeaders(headerFixture).Count!=1)throw new Exception("header-fixture");
        Array.Copy(BitConverter.GetBytes(100000UL),0,headerFixture,0x220,8);if(FindHeaders(headerFixture).Count!=0)throw new Exception("header-bounds");
        IntPtr data=Marshal.AllocHGlobal(32),handle=IntPtr.Zero;
        try {
            // Test fixture is in THIS helper's memory. Nothing is allocated/written in another process.
            byte[] b=new byte[32];Array.Copy(BitConverter.GetBytes(1.5f),0,b,0,4);Array.Copy(BitConverter.GetBytes(-1.25f),0,b,8,4);Array.Copy(BitConverter.GetBytes((short)7),0,b,12,2);Array.Copy(BitConverter.GetBytes((short)-8),0,b,14,2);Array.Copy(BitConverter.GetBytes(1f),0,b,16,4);Marshal.Copy(b,0,data,b.Length);
            handle=OpenProcess(ReadRights,false,Process.GetCurrentProcess().Id);if(handle==IntPtr.Zero)throw new Exception("self-open");
            var memory=new Memory(handle);var point=Placement(memory.Read(data.ToInt64(),32));if((double)point["x"]!=3585.5||(double)point["z"]!=-4097.25)throw new Exception("sector-conversion");
            bool refused=false;try{memory.Read(1,8);}catch{refused=true;}if(!refused)throw new Exception("range-gate");
            Emit(new {status="self-test-passed",target="helper-own-memory-only",rights=ReadRights,mbiBytes=48,gameAccess=false});return 0;
        }finally{if(handle!=IntPtr.Zero)CloseHandle(handle);Marshal.FreeHGlobal(data);}
    }
    static int TrafficKind(Memory memory,long obj,long image){
        try {
            long table=memory.Ptr(obj),function=memory.Ptr(table+8);
            uint nt=BitConverter.ToUInt32(memory.Read(image+0x3c,4),0);
            if(nt>4096)return -1;
            uint size=BitConverter.ToUInt32(memory.Read(image+nt+80,4),0);
            if(function<image||function>image+size-16)return -1;
            byte[] code=memory.Read(function,16);
            if(code[0]==0xb8&&code[5]==0xc3){int kind=BitConverter.ToInt32(code,1);return kind>=1&&kind<=7?kind:-1;}
            return -1;
        }catch{return -1;}
    }
    static void Discover(Memory memory,long image){
        long traffic=memory.Ptr(image+TrafficSlotRva);byte[] prefix=memory.Read(traffic,0x800);
        var candidates=new List<object>();
        for(int dataOffset=0x40;dataOffset<=0x7d8;dataOffset+=8){
            long data=BitConverter.ToInt64(prefix,dataOffset);ulong count=BitConverter.ToUInt64(prefix,dataOffset+8),capacity=BitConverter.ToUInt64(prefix,dataOffset+16);
            if(!Valid(data)||(data&7)!=0||count==0||count>128||capacity<count||capacity>4096)continue;
            try {
                byte[] items=memory.Read(data,(int)count*8);var kinds=new List<int>();
                for(int i=0;i<Math.Min((int)count,4);i++){long item=BitConverter.ToInt64(items,i*8);kinds.Add(TrafficKind(memory,item,image));}
                candidates.Add(new {dataFieldOffset="0x"+dataOffset.ToString("x"),count=count,capacity=capacity,firstKinds=kinds});
            }catch{}
        }
        Emit(new {status="header-discovery",trafficType=memory.TypeName(traffic,image),candidates=candidates,layoutVerified=false,onlinePlayersVerified=false});
    }
    static void TracePlayerLists(Memory memory,long image,int pid,int frame,long elapsed){
        var timer=Stopwatch.StartNew();long traffic=memory.Ptr(image+TrafficSlotRva);
        var lists=new List<object>();int inspectedVehicles=0;
        foreach(var header in FindHeaders(memory.Read(traffic,0x800))){
            try {
                byte[] pointers=memory.Read(header.Data,(int)header.Count*8);
                long first=BitConverter.ToInt64(pointers,0);if(TrafficKind(memory,first,image)!=2)continue;
                if(inspectedVehicles+(int)header.Count>256)throw new Exception("trace-row-budget");
                inspectedVehicles+=(int)header.Count;var vehicles=new List<object>();var seen=new HashSet<long>();
                for(int i=0;i<(int)header.Count;i++){
                    long vehicle=BitConverter.ToInt64(pointers,i*8);
                    if(!Valid(vehicle)||(vehicle&7)!=0||TrafficKind(memory,vehicle,image)!=2)throw new Exception("mixed-list-types");
                    if(!seen.Add(vehicle))continue;
                    var placement=Placement(memory.Read(vehicle+0x28,32));
                    placement["id"]=Digest(Encoding.ASCII.GetBytes(Salt+vehicle.ToString("x",CultureInfo.InvariantCulture))).Substring(0,12);
                    placement["classification"]="unverified-player-vehicle-list-candidate";vehicles.Add(placement);
                }
                if(memory.Ptr(traffic+header.Offset)!=header.Data||memory.U64(traffic+header.Offset+8)!=header.Count||memory.U64(traffic+header.Offset+16)!=header.Capacity)throw new Exception("snapshot-changed");
                lists.Add(new {dataFieldOffset="0x"+header.Offset.ToString("x"),tentative=header.Offset!=0x218,count=header.Count,vehicles=vehicles});
            }catch(Exception ex){
                lists.Add(new {dataFieldOffset="0x"+header.Offset.ToString("x"),status="list-refused",reason=SafeReason(ex),vehicles=new object[0]});
            }
        }
        if(memory.Ptr(image+TrafficSlotRva)!=traffic)throw new Exception("snapshot-changed");
        object own=OwnPose(pid);
        Emit(new {status="list-discovery-sample",frame=frame,elapsedMs=elapsed,readMs=timer.Elapsed.TotalMilliseconds,lists=lists,ownTelemetry=own,layoutVerified=false,completeRosterVerified=false});
    }
    static bool ParentAlive(Func<bool> exited,Func<long> creation,long expected) {
        try{return !exited()&&creation()==expected;}catch{return false;}
    }
    sealed class ParentLifetime : IDisposable {
        readonly Process parent;readonly long creation;
        public ParentLifetime(int pid) {
            if(pid<=0)throw new Exception("invalid-parent-pid");
            parent=Process.GetProcessById(pid);
            try{
                // Cache an actual process handle before polling. A recycled PID cannot retarget it.
                IntPtr stableHandle=parent.Handle;if(stableHandle==IntPtr.Zero)throw new Exception("parent-unavailable");
                creation=parent.StartTime.ToUniversalTime().Ticks;
                if(!Alive())throw new Exception("parent-exited");
            }catch{parent.Dispose();throw;}
        }
        public bool Alive(){return ParentAlive(delegate{return parent.HasExited;},delegate{return parent.StartTime.ToUniversalTime().Ticks;},creation);}
        public void Dispose(){parent.Dispose();}
    }
    static int Follow(int pid,int parentPid) {
        if(parentPid==pid||parentPid==Process.GetCurrentProcess().Id)throw new Exception("invalid-parent-pid");
        using(var parent=new ParentLifetime(parentPid))return Observe(pid,1,false,false,true,parent);
    }
    static int Observe(int pid,int seconds,bool discover,bool trace,bool follow,ParentLifetime parent){
        if(seconds<1||seconds>120||!Environment.Is64BitProcess)throw new Exception("invalid-duration-or-architecture");
        var process=Process.GetProcessById(pid);if(!String.Equals(process.ProcessName,"eurotrucks2",StringComparison.OrdinalIgnoreCase))throw new Exception("wrong-process");
        IntPtr handle=OpenProcess(ReadRights,false,pid);if(handle==IntPtr.Zero)throw new Exception("read-access-denied");
        try {
            var path=new StringBuilder(32768);uint size=(uint)path.Capacity;
            if(!QueryFullProcessImageName(handle,0,path,ref size))throw new Exception("image-path-unavailable");
            string hash;using(var sha=SHA256.Create())using(var file=File.OpenRead(path.ToString()))hash=BitConverter.ToString(sha.ComputeHash(file)).Replace("-","").ToLowerInvariant();
            if(hash!=ExpectedHash)throw new Exception("unsupported-image-fingerprint");
            var modules=new IntPtr[1024];uint needed;if(!EnumProcessModulesEx(handle,modules,(uint)(modules.Length*IntPtr.Size),out needed,2)||needed==0)throw new Exception("module-unavailable");
            long image=modules[0].ToInt64();var memory=new Memory(handle);byte[] signature=memory.Read(image+SignatureRva,Pattern.Length);
            for(int i=0;i<Pattern.Length;i++)if(Pattern[i]>=0&&Pattern[i]!=signature[i])throw new Exception("loaded-signature-mismatch");
            if(SignatureRva+10+BitConverter.ToInt32(signature,6)!=TrafficSlotRva)throw new Exception("locator-mismatch");
            bool nameSignatures=NameSignatures(memory,image);
            Emit(new {status="observation-start",mode=follow?"automatic-read-only":"research-read-only",buildSha256=hash,layoutVerified=false,onlinePlayersVerified=false,identityVerified=false,nameSignaturesMatched=nameSignatures,samplesPerSecond=follow?4:2});
            if(discover){Discover(memory,image);return 0;}
            var watch=Stopwatch.StartNew();int frame=0;
            while(follow||watch.Elapsed.TotalSeconds<seconds){
                if(follow&&(parent==null||!parent.Alive())){Emit(new {status="parent-exited",vehicles=new object[0]});break;}
                uint exitCode;if(!GetExitCodeProcess(handle,out exitCode)||exitCode!=259){Emit(new {status="target-exited",vehicles=new object[0]});break;}
                var sample=Stopwatch.StartNew();
                try {
                    if(trace){TracePlayerLists(memory,image,pid,frame,watch.ElapsedMilliseconds);frame++;Thread.Sleep(500);continue;}
                    long traffic=memory.Ptr(image+TrafficSlotRva),array=traffic+0x210;
                    ulong count=memory.U64(array+16),capacity=memory.U64(array+24);
                    if(count>128||capacity<count||capacity>4096)throw new Exception("array-bounds");
                    var vehicles=new List<object>();var placements=new Dictionary<long,Dictionary<string,object>>();var types=new Dictionary<string,int>();long list=count>0?memory.Ptr(array+8):0;
                    byte[] pointers=count>0?memory.Read(list,(int)count*8):new byte[0];var seen=new HashSet<long>();
                    for(int i=0;i<(int)count;i++){
                        long vehicle=BitConverter.ToInt64(pointers,i*8);if(!Valid(vehicle)||(vehicle&7)!=0)throw new Exception("invalid-vehicle-pointer");
                        if(TrafficKind(memory,vehicle,image)!=2)throw new Exception("unexpected-traffic-type");
                        long obj=vehicle;if(!seen.Add(obj))continue;
                        string type=memory.TypeName(obj,image);if(!types.ContainsKey(type))types[type]=0;types[type]++;
                        var placement=Placement(memory.Read(vehicle+0x28,32));placement["id"]=Digest(Encoding.ASCII.GetBytes(Salt+obj.ToString("x",CultureInfo.InvariantCulture))).Substring(0,12);
                        placement["objectType"]=type;placement["classification"]="traffic-player-vehicle-candidate";vehicles.Add(placement);placements.Add(vehicle,placement);
                    }
                    if(memory.Ptr(image+TrafficSlotRva)!=traffic||memory.U64(array+16)!=count||(count>0&&memory.Ptr(array+8)!=list))throw new Exception("snapshot-changed");
                    string nameStatus=nameSignatures?"available":"signature-mismatch";int named=0;
                    if(nameSignatures){try{var names=ResolveNames(memory.Read,image,seen);foreach(var pair in names){Dictionary<string,object> placement;if(placements.TryGetValue(pair.Key,out placement)){placement["name"]=pair.Value;placement["nameBinding"]="native-session-actor";placement["identityVerified"]=false;named++;}}}catch(Exception ex){nameStatus=SafeReason(ex);}}
                    // A name scan may take time: recheck all traffic membership before publishing it.
                    if(memory.Ptr(image+TrafficSlotRva)!=traffic||memory.U64(array+16)!=count||memory.U64(array+24)!=capacity||(count>0&&memory.Ptr(array+8)!=list))throw new Exception("snapshot-changed");
                    byte[] membership=count>0?memory.Read(list,(int)count*8):new byte[0];for(int i=0;i<pointers.Length;i++)if(pointers[i]!=membership[i])throw new Exception("snapshot-changed");
                    Emit(new {status="candidate-sample",frame=frame,elapsedMs=watch.ElapsedMilliseconds,readMs=sample.Elapsed.TotalMilliseconds,arrayCount=count,types=types,vehicles=vehicles,ownTelemetry=OwnPose(pid),layoutVerified=false,onlinePlayersVerified=false,identityVerified=false,nameStatus=nameStatus,namedPlayers=named});
                }catch(Exception ex){Emit(new {status="sample-refused",frame=frame,elapsedMs=watch.ElapsedMilliseconds,reason=SafeReason(ex),vehicles=new object[0],layoutVerified=false});}
                frame++;Thread.Sleep(follow?250:500);
            }
            Emit(new {status="observation-finished",frames=frame});return 0;
        }finally{CloseHandle(handle);}
    }
    public static int Main(string[] args){
        try {
            if(args.Length==1&&args[0]=="--self-test")return SelfTest();
            if(args.Length==5&&args[0]=="--observe"&&args[1]=="--pid"&&args[3]=="--seconds")return Observe(Int32.Parse(args[2],CultureInfo.InvariantCulture),Int32.Parse(args[4],CultureInfo.InvariantCulture),false,false,false,null);
            if(args.Length==5&&args[0]=="--follow"&&args[1]=="--pid"&&args[3]=="--parent-pid")return Follow(Int32.Parse(args[2],CultureInfo.InvariantCulture),Int32.Parse(args[4],CultureInfo.InvariantCulture));
            if(args.Length==3&&args[0]=="--discover"&&args[1]=="--pid")return Observe(Int32.Parse(args[2],CultureInfo.InvariantCulture),1,true,false,false,null);
            if(args.Length==5&&args[0]=="--trace-lists"&&args[1]=="--pid"&&args[3]=="--seconds")return Observe(Int32.Parse(args[2],CultureInfo.InvariantCulture),Int32.Parse(args[4],CultureInfo.InvariantCulture),false,true,false,null);
            Emit(new {status="refused",reason="explicit-self-test-observe-or-follow-required"});return 2;
        }catch(Exception ex){Emit(new {status="refused",reason=SafeReason(ex)});return 1;}
    }
}
