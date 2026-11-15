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
            using(var map=MemoryMappedFile.OpenExisting("Local\\WestPlay.Bridge.v28",MemoryMappedFileRights.Read))
            using(var view=map.CreateViewAccessor(0,128,MemoryMappedFileAccess.Read)){
                if(view.ReadUInt32(0)!=0x37425057||view.ReadUInt32(4)!=28||view.ReadUInt32(8)!=(uint)pid){OwnFreshness.Reset();return null;}
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
    static int SelfTest(){
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
    static int Observe(int pid,int seconds,bool discover,bool trace){
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
            Emit(new {status="observation-start",mode="research-read-only",buildSha256=hash,layoutVerified=false,onlinePlayersVerified=false,samplesPerSecond=2});
            if(discover){Discover(memory,image);return 0;}
            var watch=Stopwatch.StartNew();int frame=0;
            while(watch.Elapsed.TotalSeconds<seconds){
                uint exitCode;if(!GetExitCodeProcess(handle,out exitCode)||exitCode!=259){Emit(new {status="target-exited",vehicles=new object[0]});break;}
                var sample=Stopwatch.StartNew();
                try {
                    if(trace){TracePlayerLists(memory,image,pid,frame,watch.ElapsedMilliseconds);frame++;Thread.Sleep(500);continue;}
                    long traffic=memory.Ptr(image+TrafficSlotRva),array=traffic+0x210;
                    ulong count=memory.U64(array+16),capacity=memory.U64(array+24);
                    if(count>128||capacity<count||capacity>4096)throw new Exception("array-bounds");
                    var vehicles=new List<object>();var types=new Dictionary<string,int>();long list=count>0?memory.Ptr(array+8):0;
                    byte[] pointers=count>0?memory.Read(list,(int)count*8):new byte[0];var seen=new HashSet<long>();
                    for(int i=0;i<(int)count;i++){
                        long vehicle=BitConverter.ToInt64(pointers,i*8);if(!Valid(vehicle)||(vehicle&7)!=0)throw new Exception("invalid-vehicle-pointer");
                        if(TrafficKind(memory,vehicle,image)!=2)throw new Exception("unexpected-traffic-type");
                        long obj=vehicle;if(!seen.Add(obj))continue;
                        string type=memory.TypeName(obj,image);if(!types.ContainsKey(type))types[type]=0;types[type]++;
                        var placement=Placement(memory.Read(vehicle+0x28,32));placement["id"]=Digest(Encoding.ASCII.GetBytes(Salt+obj.ToString("x",CultureInfo.InvariantCulture))).Substring(0,12);
                        placement["objectType"]=type;placement["classification"]="traffic-player-vehicle-candidate";vehicles.Add(placement);
                    }
                    if(memory.Ptr(image+TrafficSlotRva)!=traffic||memory.U64(array+16)!=count||(count>0&&memory.Ptr(array+8)!=list))throw new Exception("snapshot-changed");
                    Emit(new {status="candidate-sample",frame=frame,elapsedMs=watch.ElapsedMilliseconds,readMs=sample.Elapsed.TotalMilliseconds,arrayCount=count,types=types,vehicles=vehicles,ownTelemetry=OwnPose(pid),layoutVerified=false,onlinePlayersVerified=false});
                }catch(Exception ex){Emit(new {status="sample-refused",frame=frame,elapsedMs=watch.ElapsedMilliseconds,reason=SafeReason(ex),vehicles=new object[0],layoutVerified=false});}
                frame++;Thread.Sleep(500);
            }
            Emit(new {status="observation-finished",frames=frame});return 0;
        }finally{CloseHandle(handle);}
    }
    public static int Main(string[] args){
        try {
            if(args.Length==1&&args[0]=="--self-test")return SelfTest();
            if(args.Length==5&&args[0]=="--observe"&&args[1]=="--pid"&&args[3]=="--seconds")return Observe(Int32.Parse(args[2],CultureInfo.InvariantCulture),Int32.Parse(args[4],CultureInfo.InvariantCulture),false,false);
            if(args.Length==3&&args[0]=="--discover"&&args[1]=="--pid")return Observe(Int32.Parse(args[2],CultureInfo.InvariantCulture),1,true,false);
            if(args.Length==5&&args[0]=="--trace-lists"&&args[1]=="--pid"&&args[3]=="--seconds")return Observe(Int32.Parse(args[2],CultureInfo.InvariantCulture),Int32.Parse(args[4],CultureInfo.InvariantCulture),false,true);
            Emit(new {status="refused",reason="explicit-self-test-or-observe-required"});return 2;
        }catch(Exception ex){Emit(new {status="refused",reason=SafeReason(ex)});return 1;}
    }
}
