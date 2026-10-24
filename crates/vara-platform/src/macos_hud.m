#import <AppKit/AppKit.h>
#import <QuartzCore/QuartzCore.h>
#import "vara_design.h"

// Native nonactivating surface. Colors are generated from the canonical DESIGN.md export.
@interface VaraPanel : NSPanel
@end
@implementation VaraPanel
-(BOOL)canBecomeKeyWindow {return NO;}
-(BOOL)canBecomeMainWindow {return NO;}
@end
static void (*cancelCallback)(void);
static void (*stopCallback)(void);
static float (*levelCallback)(void);
@interface VaraHUDControl : NSObject
-(void)cancel:(id)sender;
-(void)stop:(id)sender;
@end
@implementation VaraHUDControl
-(void)cancel:(id)sender {(void)sender;if(cancelCallback)cancelCallback();}
-(void)stop:(id)sender {(void)sender;if(stopCallback)stopCallback();}
@end
@interface VaraLevelView : NSView
@property BOOL recording;
@end
@implementation VaraLevelView
-(void)drawRect:(NSRect)dirty {
    (void)dirty;
    float level=self.recording&&levelCallback?fminf(1,fmaxf(0,levelCallback())):0;
    CGFloat envelope[]={.14,.25,.42,.65,.82,1,.72,.92,.75,.59,.38,.25,.4,.23,.13,.08};
    CGFloat center=NSMidY(self.bounds);
    for(int i=0;i<16;i++){
        CGFloat height=3+42*level*envelope[i];
        [[VARA_COLOR_PRIMARY colorWithAlphaComponent:self.recording?(i>11?.35:1):.25] setFill];
        [[NSBezierPath bezierPathWithRoundedRect:NSMakeRect(3+i*5,center-height/2,3,height) xRadius:1.5 yRadius:1.5] fill];
    }
}
@end
static VaraPanel *panel;
static NSTextField *originLabel,*stageLabel,*detailLabel;
static NSImageView *originIcon;
static VaraLevelView *levelView;
static NSButton *stopButton;
static VaraHUDControl *control;
static NSTimer *meterTimer;
static NSTextField *label(NSRect frame,CGFloat size,NSFontWeight weight,NSColor *color){
    NSTextField *field=[NSTextField labelWithString:@""];field.frame=frame;field.font=[NSFont systemFontOfSize:size weight:weight];field.textColor=color;field.lineBreakMode=NSLineBreakByTruncatingTail;return field;
}
static NSButton *actionButton(NSString *symbol,NSString *name,SEL action,NSRect frame){
    NSButton *button=[NSButton buttonWithImage:[NSImage imageWithSystemSymbolName:symbol accessibilityDescription:name] target:control action:action];button.frame=frame;button.bordered=NO;button.toolTip=name;button.accessibilityLabel=name;button.contentTintColor=VARA_COLOR_PRIMARY;button.wantsLayer=YES;button.layer.cornerRadius=16;return button;
}
void vara_hud(const char *appId,const char *appName,const char *title,const char *detail,int recording,void (*cancel)(void),void (*stop)(void),float (*level)(void)){
    NSString *identifier=[NSString stringWithUTF8String:appId],*name=[NSString stringWithUTF8String:appName],*stage=[NSString stringWithUTF8String:title],*secondary=[NSString stringWithUTF8String:detail];
    dispatch_async(dispatch_get_main_queue(),^{
        cancelCallback=cancel;stopCallback=stop;levelCallback=level;
        if(!panel){
            panel=[[VaraPanel alloc] initWithContentRect:NSMakeRect(0,0,520,76) styleMask:NSWindowStyleMaskBorderless|NSWindowStyleMaskNonactivatingPanel backing:NSBackingStoreBuffered defer:NO];
            panel.level=NSFloatingWindowLevel;panel.opaque=NO;panel.backgroundColor=NSColor.clearColor;panel.hasShadow=YES;panel.hidesOnDeactivate=NO;panel.collectionBehavior=NSWindowCollectionBehaviorCanJoinAllSpaces|NSWindowCollectionBehaviorFullScreenAuxiliary;panel.accessibilityLabel=@"Vara diktering";
            NSView *body=[[NSView alloc] initWithFrame:NSMakeRect(0,0,520,76)];body.wantsLayer=YES;body.layer.backgroundColor=VARA_COLOR_SURFACE.CGColor;body.layer.cornerRadius=24;body.layer.borderWidth=1;body.layer.borderColor=VARA_COLOR_DIVIDER.CGColor;panel.contentView=body;
            originIcon=[[NSImageView alloc] initWithFrame:NSMakeRect(20,26,24,24)];originIcon.imageScaling=NSImageScaleProportionallyUpOrDown;originIcon.accessibilityElement=NO;[body addSubview:originIcon];
            originLabel=label(NSMakeRect(54,27,96,22),14,NSFontWeightMedium,VARA_COLOR_ON_SURFACE);[body addSubview:originLabel];
            levelView=[[VaraLevelView alloc] initWithFrame:NSMakeRect(166,14,84,48)];levelView.accessibilityElement=NO;[body addSubview:levelView];
            stageLabel=label(NSMakeRect(266,41,162,19),14,NSFontWeightMedium,VARA_COLOR_ON_SURFACE);[body addSubview:stageLabel];
            detailLabel=label(NSMakeRect(266,20,162,19),13,NSFontWeightRegular,VARA_COLOR_MUTED);[body addSubview:detailLabel];
            control=[VaraHUDControl new];stopButton=actionButton(@"stop.fill",@"Afslut diktering",@selector(stop:),NSMakeRect(438,22,32,32));stopButton.layer.backgroundColor=VARA_COLOR_SURFACE_MUTED.CGColor;[body addSubview:stopButton];
            [body addSubview:actionButton(@"xmark",@"Annuller diktering",@selector(cancel:),NSMakeRect(478,22,32,32))];
        }
        NSURL *application=[NSWorkspace.sharedWorkspace URLForApplicationWithBundleIdentifier:identifier];originIcon.image=application?[NSWorkspace.sharedWorkspace iconForFile:application.path]:[NSImage imageWithSystemSymbolName:@"app" accessibilityDescription:nil];
        originLabel.stringValue=name;originLabel.toolTip=name;stageLabel.stringValue=stage;detailLabel.stringValue=secondary;detailLabel.toolTip=secondary;stopButton.hidden=!recording;levelView.recording=recording;
        NSScreen *screen=NSScreen.mainScreen;NSRect frame=screen.visibleFrame;[panel setFrameOrigin:NSMakePoint(NSMidX(frame)-260,NSMinY(frame)+36)];[panel orderFrontRegardless];
        [meterTimer invalidate];meterTimer=[NSTimer scheduledTimerWithTimeInterval:.06 repeats:YES block:^(NSTimer *timer){(void)timer;levelView.needsDisplay=YES;}];
    });
}
void vara_hide_hud(void){dispatch_async(dispatch_get_main_queue(),^{[meterTimer invalidate];meterTimer=nil;[panel orderOut:nil];cancelCallback=NULL;stopCallback=NULL;levelCallback=NULL;});}
